use bytes::Bytes;


use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::{
    docstore::{DocumentStore, DocumentStoreKind},
    formatter::{CitationStyle, Cite, DocumentFormatter, TextFormatter},
    index::{FaceIndex, SearchService},
    openai::{LanguageServiceArguments, LlmMessage, LlmRole, OpenAiDelegate, PartialLlmMessage},
    server::{Conversation, CountSources, Message, PartialMessage, Source},
};

use super::QueryEngineError;

pub struct Engine {
    index: FaceIndex,
    openai: OpenAiDelegate,
    docstore: DocumentStoreKind,
    system_prompt: String,
}

impl Engine {
    pub(crate) fn new(
        index: FaceIndex,
        openai: OpenAiDelegate,
        docstore: DocumentStoreKind,
        system_prompt: String,
    ) -> Self {
        Self {
            index,
            openai,
            docstore,
            system_prompt,
        }
    }
}

const NUM_DOCUMENTS_TO_RETRIEVE: usize = 4;

const CITATION_STYLE: CitationStyle = CitationStyle::MLA;

impl Engine {
    pub(crate) async fn query(&self, question: &str) -> Result<String, QueryEngineError> {
        let (_, formatted_documents) = self.get_documents(question, 0usize).await?;

        Ok(formatted_documents)
    }

    pub(crate) async fn conversation(
        &self,
        Conversation { messages }: Conversation,
        stop_phrases: Vec<&str>,
    ) -> Result<Message, QueryEngineError> {
        let num_sources = messages.sources_count();
        match messages.into_iter().last() {
            Some(Message::User(user_query)) => {
                let (sources, formatted_documents) =
                    self.get_documents(&user_query, num_sources).await?;

                let llm_service_arguments = LanguageServiceArguments {
                    system: &self.system_prompt,
                    documents: &formatted_documents,
                    query: &user_query,
                    citation_index_begin: num_sources,
                };

                let LlmMessage { role, content } = self
                    .openai
                    .get_llm_answer(llm_service_arguments, 2048u16, stop_phrases)
                    .await
                    .map_err(QueryEngineError::LlmError)?;

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
    pub(crate) async fn streaming_conversation(
        &self,
        Conversation { messages }: Conversation,
        tx: UnboundedSender<Bytes>,
        stop_phrases: Vec<&str>,
    ) -> Result<(), QueryEngineError> {
        let num_sources = messages.sources_count();
        match messages.into_iter().last() {
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
                let llm_service_arguments = LanguageServiceArguments {
                    system: &self.system_prompt,
                    documents: &formatted_documents,
                    query: &user_query,
                    citation_index_begin: num_sources,
                };
                self.openai
                    .stream_llm_answer(llm_service_arguments, tx_p, 2048u16, stop_phrases)
                    .await
                    .map_err(QueryEngineError::LlmError)?;

                Ok(())
            }
            Some(Message::Assistant(_, _)) => Err(QueryEngineError::LastMessageIsNotUser)?,
            None => Err(QueryEngineError::EmptyConversation)?,
        }
    }

    pub(crate) async fn get_documents(
        &self,
        user_query: &str,
        num_sources_already_in_chat: usize,
    ) -> Result<(Vec<Source>, String), QueryEngineError> {
        let embedding = self
            .openai
            .embed(user_query)
            .await
            .map_err(QueryEngineError::EmbeddingServiceError)?;

        let document_indices = self
            .index
            .search(embedding, NUM_DOCUMENTS_TO_RETRIEVE)
            .await
            .map_err(QueryEngineError::IndexError)?;

        let documents = self
            .docstore
            .retreive(&document_indices)
            .await
            .map_err(QueryEngineError::DocstoreError)?;

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
