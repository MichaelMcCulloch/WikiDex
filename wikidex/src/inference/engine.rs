use bytes::Bytes;

use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::{
    docstore::{Document, DocumentStore, DocumentStoreImpl},
    embedding_client::{EmbeddingClient, EmbeddingClientService},
    formatter::{CitationStyle, Cite},
    index::{FaceIndex, SearchService},
    llm_client::{
        LanguageServiceArguments, LanguageServiceDocument, LlmClientImpl, LlmClientService,
        LlmMessage, LlmRole, PartialLlmMessage,
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

        let document_arguments = documents
            .iter()
            .map(|d| LanguageServiceDocument {
                index: d.index,
                text: d.text.clone(),
            })
            .collect::<Vec<_>>();

        let sources = documents
            .into_iter()
            .enumerate()
            .map(|(ordinal, document)| Source {
                ordinal: ordinal + num_sources,
                index: document.index,
                citation: document.provenance.format(&CITATION_STYLE),
                url: document.provenance.url(),
                origin_text: document.text,
            })
            .collect::<Vec<_>>();

        let llm_service_arguments = LanguageServiceArguments {
            messages,
            documents: document_arguments,
            user_query,
        };
        let LlmMessage { role, content } = self
            .llm_client
            .get_llm_answer(llm_service_arguments, 2048u16, stop_phrases)
            .await?;

        match role {
            LlmRole::Assistant => {
                let content = content.trim().to_string();
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
        let _num_sources = messages.sources_count();
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

        let document_arguments = documents
            .iter()
            .map(|d| LanguageServiceDocument {
                index: d.index,
                text: d.text.clone(),
            })
            .collect::<Vec<_>>();
        let llm_service_arguments = LanguageServiceArguments {
            messages,
            documents: document_arguments,
            user_query,
        };
        let _documents = documents.into_iter().map(Some).collect::<Vec<_>>();

        let (partial_message_sender, mut partial_message_receiver) = unbounded_channel();

        actix_web::rt::spawn(async move {
            while let Some(PartialLlmMessage {
                content: Some(content),
                ..
            }) = partial_message_receiver.recv().await
            {
                let _ = tx.send(PartialMessage::content(content).message());
            }

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
