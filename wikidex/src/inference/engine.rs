use bytes::Bytes;
use chrono::{DateTime, Utc};

use std::{
    collections::HashMap,
    ops::DerefMut,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};
use tera::{Context, Tera};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedSender},
    time::sleep,
};

use crate::{
    docstore::{Document, DocumentStore, DocumentStoreImpl},
    embedding_client::{EmbeddingClient, EmbeddingClientService},
    formatter::{CitationStyle, Cite},
    index::{FaceIndex, SearchService},
    llm_client::{
        LanguageServiceArguments, LlmClientImpl, LlmClientService, LlmMessage, LlmRole,
        PartialLlmMessage,
    },
    server::{Conversation, CountSources, Message, PartialMessage, Source},
};

use super::QueryEngineError;

pub struct Engine {
    index: FaceIndex,
    embed_client: EmbeddingClient,
    docstore: DocumentStoreImpl,
    llm_client: LlmClientImpl,
    system_prompt: Arc<RwLock<Tera>>,
}

impl Engine {
    pub(crate) async fn new(
        index: FaceIndex,
        embed_client: EmbeddingClient,
        llm_client: LlmClientImpl,
        docstore: DocumentStoreImpl,
        system_prompt: PathBuf,
    ) -> Self {
        let system_prompt = Arc::new(RwLock::new(
            Tera::new(system_prompt.to_str().unwrap()).unwrap(),
        ));
        let tera = system_prompt.clone();
        actix_web::rt::spawn(async move {
            loop {
                sleep(Duration::from_secs(2)).await;
                tera.write()
                    .map(|mut t| match t.deref_mut().full_reload() {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!("Could Not Reload Template! {e}");
                        }
                    })
                    .unwrap();
            }
        });
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
    pub(crate) async fn conversation(
        &self,
        Conversation { messages }: Conversation,
        stop_phrases: Vec<&str>,
    ) -> Result<Message, QueryEngineError> {
        let num_sources = messages.sources_count();
        match messages.into_iter().last() {
            Some(Message::User(user_query)) => {
                let documents = self.get_documents(&user_query).await?;
                let prompt = self.format_rag_template(&documents, &user_query)?;
                let sources = organize_sources(documents, num_sources);

                let llm_service_arguments = LanguageServiceArguments {
                    prompt: &prompt,
                    query: &user_query,
                    indices: &sources.iter().map(|d| d.index).collect(),
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
                let documents = self.get_documents(&user_query).await?;
                let prompt = self.format_rag_template(&documents, &user_query)?;

                let sources = organize_sources(documents, num_sources);

                let (partial_message_sender, mut partial_message_receiver) = unbounded_channel();

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
                    }) = partial_message_receiver.recv().await
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

                    let _ = tx.send(PartialMessage::done().message());
                });

                let llm_service_arguments = LanguageServiceArguments {
                    prompt: &prompt,
                    query: &user_query,
                    indices: &sources.iter().map(|d| d.index).collect(),
                };
                self.llm_client
                    .stream_llm_answer(
                        llm_service_arguments,
                        partial_message_sender,
                        2048u16,
                        stop_phrases,
                    )
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
    ) -> Result<Vec<Document>, QueryEngineError> {
        let embedding: Vec<f32> = self.embed_client.embed(user_query).await?;

        let document_indices = self
            .index
            .search(embedding, NUM_DOCUMENTS_TO_RETRIEVE)
            .await?;

        let documents = self.docstore.retreive(&document_indices).await?;

        Ok(documents)
    }
    fn format_rag_template(
        &self,
        documents: &Vec<Document>,
        user_query: &str,
    ) -> Result<String, QueryEngineError> {
        let mut context = Context::new();
        context.insert("document_list", documents);
        context.insert("user_query", user_query);
        context.insert(
            "current_time",
            &DateTime::<Utc>::from(SystemTime::now()).to_rfc3339(),
        );
        let prompt = self
            .system_prompt
            .read()
            .unwrap()
            .render("instruct/markdown.md.j2", &context)?;
        Ok(prompt)
    }
}

fn organize_sources(documents: Vec<Document>, num_sources: usize) -> Vec<Source> {
    documents
        .into_iter()
        .enumerate()
        .map(|(ordinal, document)| Source {
            ordinal: num_sources + ordinal + 1,
            index: document.index,
            citation: document.provenance.format(&CITATION_STYLE),
            url: document.provenance.url(),
            origin_text: document.text,
        })
        .collect::<Vec<_>>()
}
