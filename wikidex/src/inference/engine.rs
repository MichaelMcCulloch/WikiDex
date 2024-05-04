use bytes::Bytes;

use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::{
    docstore::{Document, DocumentStore, DocumentStoreImpl},
    embedding_client::{EmbeddingClient, EmbeddingClientService},
    formatter::{CitationStyle, Cite},
    index::{FaceIndex, SearchService},
    inference::index_accumulator::IndexAccumulator,
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
}

impl Engine {
    pub(crate) async fn new(
        index: FaceIndex,
        embed_client: EmbeddingClient,
        llm_client: LlmClientImpl,
        docstore: DocumentStoreImpl,
    ) -> Self {
        Self {
            index,
            embed_client,
            docstore,
            llm_client,
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

        let user_query = match messages.iter().last() {
            Some(Message::User(user_query)) => {
                Ok::<std::string::String, QueryEngineError>(user_query.clone())
            }
            Some(Message::Assistant(_, _)) => Err(QueryEngineError::LastMessageIsNotUser)?,
            None => Err(QueryEngineError::EmptyConversation)?,
        }?;

        let messages = messages
            .into_iter()
            .map(|m| match m {
                Message::User(content) => LlmMessage {
                    role: LlmRole::User,
                    content,
                },
                Message::Assistant(content, _) => LlmMessage {
                    role: LlmRole::Assistant,
                    content,
                },
            })
            .collect::<Vec<_>>();

        let documents = self.get_documents(&user_query).await?;
        log::info!("User message: \"{user_query}\"",);
        log::info!(
            "Obtained documents:\n{}.",
            documents
                .iter()
                .map(|d| format!("{}:{}", d.index, d.text.lines().next().unwrap()))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let llm_service_arguments = LanguageServiceArguments {
            messages,
            documents: documents.clone(),
            user_query,
        };
        let sources = organize_sources(documents, num_sources);
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
                        format!("[{}](http://localhost/#{})", source.ordinal, source.ordinal)
                            .as_str(),
                    );
                }

                Ok(Message::Assistant(content, sources))
            }
            _ => Err(QueryEngineError::InvalidAgentResponse)?,
        }
    }

    pub(crate) async fn streaming_conversation(
        &self,
        Conversation { messages }: Conversation,
        tx: UnboundedSender<Bytes>,
        stop_phrases: Vec<&str>,
    ) -> Result<(), QueryEngineError> {
        let num_sources = messages.sources_count();
        let user_query = match messages.iter().last() {
            Some(Message::User(user_query)) => {
                Ok::<std::string::String, QueryEngineError>(user_query.clone())
            }
            Some(Message::Assistant(_, _)) => Err(QueryEngineError::LastMessageIsNotUser)?,
            None => Err(QueryEngineError::EmptyConversation)?,
        }?;
        let messages = messages
            .into_iter()
            .map(|m| match m {
                Message::User(content) => LlmMessage {
                    role: LlmRole::User,
                    content,
                },
                Message::Assistant(content, _) => LlmMessage {
                    role: LlmRole::Assistant,
                    content,
                },
            })
            .collect::<Vec<_>>();

        let documents = self.get_documents(&user_query).await?;
        let dictionary = documents
            .iter()
            .map(|Document { index, .. }| *index)
            .collect::<Vec<_>>(); // Sample dictionary

        // Define a formatter function
        let formatter = Box::new(|index: usize| format!("[{index}](http://localhost/#{index})"));

        // Create a new IndexAccumulator
        let _accumulator = IndexAccumulator::new(dictionary, formatter);

        log::info!("User message: \"{user_query}\"",);
        log::info!(
            "Obtained documents:\n{}.",
            documents
                .iter()
                .map(|d| format!("{}:{}", d.index, d.text.lines().next().unwrap()))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let llm_service_arguments = LanguageServiceArguments {
            messages,
            documents: documents.clone(),
            user_query,
        };
        let sources = organize_sources(documents, num_sources);

        let (partial_message_sender, mut partial_message_receiver) = unbounded_channel();

        let _sources_list = sources.clone();
        actix_web::rt::spawn(async move {
            // let mut accumulated_index = String::new();
            // let mut accumulating_index = false;
            // let mut index_ordinal_map = HashMap::new();
            // let mut send_message = |accumulated_index: String| {
            //     let index = accumulated_index.trim().parse::<i64>().unwrap();

            //     if let Some(source) = sources_list
            //         .iter()
            //         .position(|s| s.index == index)
            //         .map(|i| sources_list.remove(i))
            //     {
            //         index_ordinal_map.insert(index, source.ordinal);
            //         let _ = tx.send(PartialMessage::source(source).message());
            //     }

            //     if let Some(ordinal) = index_ordinal_map.get(&index) {
            //         let source_link = accumulated_index.replace(
            //             accumulated_index.as_str(),
            //             format!("[{ordinal}](http://localhost/#{ordinal})").as_str(),
            //         );
            //         let _ = tx.send(PartialMessage::content(source_link).message());
            //     } else {
            //         let _ = tx.send(PartialMessage::content(accumulated_index).message());
            //     }
            // };

            while let Some(PartialLlmMessage {
                content: Some(content),
                ..
            }) = partial_message_receiver.recv().await
            {
                let _ = tx.send(PartialMessage::content(content).message());
                // Check if the token is numeric (ignoring any leading/trailing whitespace)
                // if content.trim().parse::<i64>().is_ok() {
                //     accumulated_index.push_str(&content);
                //     accumulating_index = true;
                // } else if accumulating_index {
                //     send_message(accumulated_index);
                //     let _ = tx.send(PartialMessage::content(content).message());
                //     accumulated_index = String::new();
                //     accumulating_index = false;
                // } else {
                //     let _ = tx.send(PartialMessage::content(content).message());
                // }
            }

            // Send any remaining accumulated number
            // if !accumulated_index.is_empty() {
            //     send_message(accumulated_index);
            // }

            let _ = tx.send(PartialMessage::done().message());
        });

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
