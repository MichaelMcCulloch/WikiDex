use std::sync::Mutex;

use bytes::Bytes;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::{
    docstore::{DocumentService, SqliteDocstore},
    embed::{r#async::Embedder, EmbedService},
    formatter::{CitationStyle, Cite, DocumentFormatter, TextFormatter},
    index::{FaissIndex, SearchService},
    llm::{
        AsyncLlmService, AsyncOpenAiService, PartialLlmMessage, {LlmInput, LlmMessage, LlmRole},
    },
    server::{Conversation, Message, PartialMessage, Source},
};

use super::{QueryEngine, QueryEngineError};

pub struct Engine {
    index: Mutex<FaissIndex>,
    embed: Embedder,
    docstore: SqliteDocstore,
    llm: AsyncOpenAiService,
    prompt: String,
}

const NUM_DOCUMENTS_TO_RETRIEVE: usize = 4;

#[async_trait::async_trait]
impl QueryEngine for Engine {
    type E = QueryEngineError;
    async fn query(&self, question: &str) -> Result<String, Self::E> {
        let embedding = self
            .embed
            .embed(&question)
            .await
            .map_err(|e| QueryEngineError::EmbeddingError(e))?;

        let result = self
            .index
            .lock()
            .map_err(|_| QueryEngineError::UnableToLockIndex)?
            .search(&embedding, NUM_DOCUMENTS_TO_RETRIEVE)
            .map_err(|e| QueryEngineError::IndexError(e))?;

        let documents = self
            .docstore
            .retreive(&result)
            .await
            .map_err(|e| QueryEngineError::DocstoreError(e))?;

        let formatted_documents = documents
            .iter()
            .map(|(index, document, _)| DocumentFormatter::format_document(*index, document))
            .collect::<Vec<String>>()
            .join("\n\n");

        Ok(formatted_documents)
    }

    async fn conversation(
        &self,
        Conversation(message_history): &Conversation,
    ) -> Result<Message, Self::E> {
        match message_history.last() {
            Some(Message::User(user_query)) => {
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
                    .map(|(index, document, _provenance)| {
                        DocumentFormatter::format_document(*index, document)
                    })
                    .collect::<Vec<String>>()
                    .join("\n\n");

                let system = self
                    .prompt
                    .replace("###DOCUMENT_LIST###", &formatted_documents)
                    .replace("###USER_QUERY###", &user_query);

                let input = LlmInput {
                    system,
                    conversation: vec![LlmMessage {
                        role: LlmRole::User,
                        content: format!("{user_query}"),
                    }],
                };

                let LlmMessage { role, content } = self
                    .llm
                    .get_llm_answer(input, None)
                    .await
                    .map_err(|e| QueryEngineError::LlmError(e))?;

                match role {
                    LlmRole::Assistant => Ok(Message::Assistant(
                        content.to_string(),
                        documents
                            .into_iter()
                            .zip(document_indices)
                            .map(|((ordinal, origin_text, provenance), index)| Source {
                                ordinal,
                                index,
                                citation: provenance.format(CitationStyle::MLA),
                                url: provenance.url(),
                                origin_text,
                            })
                            .collect(),
                    )),
                    _ => Err(QueryEngineError::InvalidAgentResponse)?,
                }
            }
            Some(Message::Assistant(_, _)) => Err(QueryEngineError::LastMessageIsNotUser)?,
            None => Err(QueryEngineError::EmptyConversation)?,
        }
    }
    async fn streaming_conversation(
        &self,
        Conversation(message_history): &Conversation,
        tx: UnboundedSender<Bytes>,
    ) -> Result<(), Self::E> {
        match message_history.last() {
            Some(Message::User(user_query)) => {
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
                    .map(|(index, document, _provenance)| {
                        DocumentFormatter::format_document(*index, document)
                    })
                    .collect::<Vec<String>>()
                    .join("\n\n");

                documents.into_iter().zip(document_indices).for_each(
                    |((ordinal, origin_text, provenance), index)| {
                        let partial_message = PartialMessage {
                            content: None,
                            source: Some(Source {
                                ordinal,
                                index,
                                citation: provenance.format(CitationStyle::MLA),
                                url: provenance.url(),
                                origin_text,
                            }),
                            finished: None,
                        };
                        let message_string = &serde_json::to_string(&partial_message).unwrap();
                        let message_bytes = Bytes::from(
                            ["event: message\ndata: ", message_string, "\n\n"].concat(),
                        );
                        let _ = tx.send(message_bytes);
                    },
                );

                let system = self
                    .prompt
                    .replace("###DOCUMENT_LIST###", &formatted_documents)
                    .replace("###USER_QUERY###", user_query);

                let input = LlmInput {
                    system,
                    conversation: vec![LlmMessage {
                        role: LlmRole::User,
                        content: format!("{user_query}"),
                    }],
                };

                let (tx_p, mut rx_p) = unbounded_channel();

                actix_web::rt::spawn(async move {
                    while let Some(PartialLlmMessage {
                        content: Some(content),
                        ..
                    }) = rx_p.recv().await
                    {
                        let partial_message = PartialMessage {
                            content: Some(content),
                            source: None,
                            finished: None,
                        };

                        let message_string = &serde_json::to_string(&partial_message).unwrap();
                        let message_bytes = Bytes::from(
                            ["event: message\ndata: ", message_string, "\n\n"].concat(),
                        );
                        let _ = tx.send(message_bytes);
                    }
                    let finished_flag = PartialMessage {
                        content: None,
                        source: None,
                        finished: Some(String::from("DONE")),
                    };
                    let finished_string = &serde_json::to_string(&finished_flag).unwrap();
                    let message_bytes =
                        Bytes::from(["event: message\ndata: ", finished_string, "\n\n"].concat());
                    let _ = tx.send(message_bytes);
                });

                self.llm
                    .stream_llm_answer(input, None, tx_p)
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
        embed: Embedder,
        docstore: SqliteDocstore,
        llm: AsyncOpenAiService,
        prompt: String,
    ) -> Self {
        Self {
            index,
            embed,
            docstore,
            llm,
            prompt,
        }
    }
}
