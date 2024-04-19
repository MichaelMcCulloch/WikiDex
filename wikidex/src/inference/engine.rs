use bytes::Bytes;
use std::collections::HashMap;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::{
    docstore::{DocumentStore, DocumentStoreKind},
    embedding_client::{EmbeddingClient, EmbeddingClientService},
    formatter::{CitationStyle, Cite, TextFormatter},
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

const CITATION_STYLE: CitationStyle = CitationStyle::Mla;

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
                    sources: &sources,
                };

                let LlmMessage { role, content } = self
                    .llm_client
                    .get_llm_answer(llm_service_arguments, 2048u16, stop_phrases)
                    .await?;

                match role {
                    LlmRole::Assistant => {
                        let mut content = content.trim().to_string();
                        for source in sources.iter() {
                            content = content.replace(
                                format!("{}", source.index).as_str(),
                                format!(
                                    "[{}](http://localhost/#{})",
                                    source.ordinal, source.ordinal
                                )
                                .as_str(),
                            );
                        }

                        Ok(Message::Assistant(content, sources))
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

                let mut sources_list = sources.clone();
                actix_web::rt::spawn(async move {
                    let mut accumulated_index = String::new();
                    let mut accumulating_index = false;
                    let mut index_ordinal_map = HashMap::new();
                    let mut send_message = |accumulated_index: String| {
                        let index = accumulated_index.trim().parse::<i64>().unwrap();

                        if let Some(source) = sources_list
                            .iter()
                            .position(|s| s.index == index)
                            .map(|i| sources_list.remove(i))
                        {
                            index_ordinal_map.insert(index, source.ordinal);
                            let _ = tx.send(PartialMessage::source(source).message());
                        }

                        if let Some(ordinal) = index_ordinal_map.get(&index) {
                            let source_link = accumulated_index.replace(
                                accumulated_index.as_str(),
                                format!("[{ordinal}](http://localhost/#{ordinal})").as_str(),
                            );
                            let _ = tx.send(PartialMessage::content(source_link).message());
                        } else {
                            let _ = tx.send(PartialMessage::content(accumulated_index).message());
                        }
                    };

                    while let Some(PartialLlmMessage {
                        content: Some(content),
                        ..
                    }) = rx_p.recv().await
                    {
                        // Check if the token is numeric (ignoring any leading/trailing whitespace)
                        if content.trim().parse::<i64>().is_ok() {
                            accumulated_index.push_str(&content);
                            accumulating_index = true;
                        } else if accumulating_index {
                            send_message(accumulated_index);
                            let _ = tx.send(PartialMessage::content(content).message());
                            accumulated_index = String::new();
                            accumulating_index = false;
                        } else {
                            let _ = tx.send(PartialMessage::content(content).message());
                        }
                    }

                    // Send any remaining accumulated number
                    if !accumulated_index.is_empty() {
                        send_message(accumulated_index);
                    }

                    log::info!("{index_ordinal_map:?}");

                    let _ = tx.send(PartialMessage::done().message());
                });
                let llm_service_arguments = LanguageServiceArguments {
                    system: &self.system_prompt,
                    documents: &formatted_documents,
                    query: &user_query,
                    sources: &sources,
                };
                self.llm_client
                    .stream_llm_answer(llm_service_arguments, tx_p, 2048u16, stop_phrases)
                    .await?;

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
        let embedding = self.embed_client.embed(user_query).await?;

        let document_indices = self
            .index
            .search(embedding, NUM_DOCUMENTS_TO_RETRIEVE)
            .await?;

        let documents = self.docstore.retreive(&document_indices).await?;

        let formatted_documents = documents
            .iter()
            .map(|document| document.format_document())
            .collect::<Vec<String>>()
            .join("\n\n");

        let sources = documents
            .into_iter()
            .enumerate()
            .map(|(ordinal, document)| Source {
                ordinal: num_sources_already_in_chat + ordinal + 1,
                index: document.index,
                citation: document.provenance.format(&CITATION_STYLE),
                url: document.provenance.url(),
                origin_text: document.text,
            })
            .collect::<Vec<_>>();
        log::info!("{sources:?}");
        Ok((sources, formatted_documents))
    }
}
