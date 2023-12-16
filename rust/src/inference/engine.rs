use std::sync::Mutex;

use bytes::Bytes;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::{
    docstore::{DocumentService, SqliteDocstore},
    embed::{
        r#async::{openai::OpenAiEmbeddingService, Embedder},
        EmbedService,
    },
    formatter::{CitationStyle, Cite, DocumentFormatter, TextFormatter},
    index::{FaissIndex, SearchService},
    llm::{
        AsyncLlmServiceArguments, LlmService, OpenAiLlmService, PartialLlmMessage,
        {LlmMessage, LlmRole},
    },
    server::{Conversation, CountSources, Message, PartialMessage, Source},
};

use super::{QueryEngine, QueryEngineError};

pub struct Engine {
    index: Mutex<FaissIndex>,
    embed: OpenAiEmbeddingService,
    docstore: SqliteDocstore,
    llm: OpenAiLlmService,
    system_prompt: String,
}

const NUM_DOCUMENTS_TO_RETRIEVE: usize = 4;

const CITATION_STYLE: CitationStyle = CitationStyle::MLA;

#[async_trait::async_trait]
impl QueryEngine for Engine {
    type E = QueryEngineError;
    async fn query(&self, question: &str) -> Result<String, Self::E> {
        let (_, formatted_documents) = self.get_documents(question, 0usize).await?;

        Ok(formatted_documents)
    }

    async fn conversation(
        &self,
        Conversation(message_history): Conversation,
    ) -> Result<Message, Self::E> {
        let num_sources = message_history.sources_count();
        match message_history.into_iter().last() {
            Some(Message::User(user_query)) => {
                let (sources, formatted_documents) =
                    self.get_documents(&user_query, num_sources).await?;

                let llm_service_arguments = AsyncLlmServiceArguments {
                    system: &self.system_prompt,
                    documents: &formatted_documents,
                    query: &user_query,
                    citation_index_begin: num_sources,
                };

                let LlmMessage { role, content } = self
                    .llm
                    .get_llm_answer(llm_service_arguments)
                    .await
                    .map_err(|e| QueryEngineError::LlmError(e))?;

                match role {
                    LlmRole::Assistant => {
                        Ok(Message::Assistant(content.trim().to_string(), sources))
                    }
                    _ => Err(QueryEngineError::InvalidAgentResponse)?,
                }
            }
            Some(Message::Assistant(_, _)) => Err(QueryEngineError::LastMessageIsNotUser)?,
            None => Err(QueryEngineError::EmptyConversation)?,
        }
    }
    async fn streaming_conversation(
        &self,
        Conversation(message_history): Conversation,
        tx: UnboundedSender<Bytes>,
    ) -> Result<(), Self::E> {
        let num_sources = message_history.sources_count();
        match message_history.into_iter().last() {
            Some(Message::User(user_query)) => {
                let (sources, formatted_documents) =
                    self.get_documents(&user_query, num_sources).await?;

                let (tx_p, mut rx_p) = unbounded_channel();

                sources.into_iter().for_each(|source| {
                    let _ = tx.send(PartialMessage::source(source).message());
                });

                actix_web::rt::spawn(async move {
                    while let Some(PartialLlmMessage {
                        content: Some(content),
                        ..
                    }) = rx_p.recv().await
                    {
                        let _ = tx.send(PartialMessage::content(content).message());
                    }
                    let _ = tx.send(PartialMessage::done().message());
                });
                let llm_service_arguments = AsyncLlmServiceArguments {
                    system: &self.system_prompt,
                    documents: &formatted_documents,
                    query: &user_query,
                    citation_index_begin: num_sources,
                };
                self.llm
                    .stream_llm_answer(llm_service_arguments, tx_p)
                    .await
                    .map_err(|e| QueryEngineError::LlmError(e))?;

                Ok(())
            }
            Some(Message::Assistant(_, _)) => Err(QueryEngineError::LastMessageIsNotUser)?,
            None => Err(QueryEngineError::EmptyConversation)?,
        }
    }
}

impl Engine {
    pub(crate) fn new(
        index: Mutex<FaissIndex>,
        embed: OpenAiEmbeddingService,
        docstore: SqliteDocstore,
        llm: OpenAiLlmService,
        system_prompt: String,
    ) -> Self {
        Self {
            index,
            embed,
            docstore,
            llm,
            system_prompt,
        }
    }

    async fn get_documents(
        &self,
        user_query: &str,
        num_sources_already_in_chat: usize,
    ) -> Result<(Vec<Source>, String), <Self as QueryEngine>::E> {
        let embedding = self
            .embed
            .embed(&user_query)
            .await
            .map_err(|e| QueryEngineError::EmbeddingError(e))?;

        let document_indices = self
            .index
            .lock()
            .map_err(|_| QueryEngineError::UnableToLockIndex)?
            .search(&embedding, NUM_DOCUMENTS_TO_RETRIEVE)
            .map_err(|e| QueryEngineError::IndexError(e))?;

        let documents = self
            .docstore
            .retreive(&document_indices)
            .await
            .map_err(|e| QueryEngineError::DocstoreError(e))?;

        let formatted_documents = documents
            .iter()
            .map(|(ordianal, document, provenance)| {
                DocumentFormatter::format_document(
                    *ordianal + num_sources_already_in_chat,
                    &provenance.title(),
                    document,
                )
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        let sources = documents
            .into_iter()
            .zip(document_indices)
            .map(|((ordinal, origin_text, provenance), index)| Source {
                ordinal: ordinal + num_sources_already_in_chat,
                index,
                citation: provenance.format(&CITATION_STYLE),
                url: provenance.url(),
                origin_text,
            })
            .collect::<Vec<_>>();

        Ok((sources, formatted_documents))
    }
}
