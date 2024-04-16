

use bytes::Bytes;

use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::{
    docstore::{DocumentStore, DocumentStoreKind},
    embedding_client::{EmbeddingClient, EmbeddingClientService},
    formatter::{CitationStyle, Cite, DocumentFormatter, TextFormatter},
    index::{FaceIndex, SearchService},
    llm_client::{
        LanguageServiceArguments, LlmClientKind, LlmClientService, LlmMessage, LlmRole,
        PartialLlmMessage,
    },
    server::{Conversation, CountSources, Message, PartialMessage, Source},
};

use super::QueryEngineError;

pub struct Engine {
    index: FaceIndex,
    embed_client: EmbeddingClient,
    docstore: DocumentStoreKind,
    llm_client: LlmClientKind,
    system_prompt: String,
}

impl Engine {
    pub(crate) fn new(
        index: FaceIndex,
        embed_client: EmbeddingClient,
        llm_client: LlmClientKind,
        docstore: DocumentStoreKind,
        system_prompt: String,
    ) -> Self {
        Self {
            index,
            embed_client,
            docstore,
            llm_client,
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
                    .llm_client
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
                self.llm_client
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
            .embed_client
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
            .map(|document| {
                DocumentFormatter::format_document(
                    document.ordinal + num_sources_already_in_chat,
                    &document.provenance.title(),
                    &document.text,
                )
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        let sources = documents
            .into_iter()
            .map(|document| Source {
                ordinal: document.ordinal + num_sources_already_in_chat,
                index: document.index,
                citation: document.provenance.format(&CITATION_STYLE),
                url: document.provenance.url(),
                origin_text: document.text,
            })
            .collect::<Vec<_>>();

        Ok((sources, formatted_documents))
    }
}
