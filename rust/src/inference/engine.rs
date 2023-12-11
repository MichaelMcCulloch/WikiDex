use std::sync::Mutex;

use bytes::Bytes;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::{
    docstore::{DocumentService, SqliteDocstore},
    embed::{r#async::Embedder, EmbedService},
    formatter::{CitationStyle, Cite, DocumentFormatter, Provenance, TextFormatter},
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
    system_prompt: String,
}

const NUM_DOCUMENTS_TO_RETRIEVE: usize = 4;

const CITATION_STYLE: CitationStyle = CitationStyle::MLA;

#[async_trait::async_trait]
impl QueryEngine for Engine {
    type E = QueryEngineError;
    async fn query(&self, question: &str) -> Result<String, Self::E> {
        let (_, _, formatted_documents) = self.get_documents(question).await?;

        Ok(formatted_documents)
    }

    async fn conversation(
        &self,
        Conversation(message_history): &Conversation,
    ) -> Result<Message, Self::E> {
        match message_history.last() {
            Some(Message::User(user_query)) => {
                let (sources, machine_input) = self
                    .prepare_answer_data(user_query, &CITATION_STYLE)
                    .await?;

                let LlmMessage { role, content } = self
                    .llm
                    .get_llm_answer(machine_input, None)
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
        Conversation(message_history): &Conversation,
        tx: UnboundedSender<Bytes>,
    ) -> Result<(), Self::E> {
        match message_history.last() {
            Some(Message::User(user_query)) => {
                let (sources, machine_input) = self
                    .prepare_answer_data(user_query, &CITATION_STYLE)
                    .await?;

                let (tx_p, mut rx_p) = unbounded_channel();

                sources.into_iter().for_each(|source| {
                    let _ = tx.send(PartialMessage::source(source).message());
                });

                actix_web::rt::spawn(async move {
                    let mut whitespace_so_far = true;
                    while let Some(PartialLlmMessage {
                        content: Some(content),
                        ..
                    }) = rx_p.recv().await
                    {
                        if whitespace_so_far
                            && !["\n\n", "\n", "\t", " "].contains(&content.as_str())
                        {
                            whitespace_so_far = false;
                        }

                        if whitespace_so_far {
                            continue;
                        }

                        // TODO: vLLM only started including the "<|assistant|>" token recently. WHY!?
                        let _ = tx.send(PartialMessage::content(content).message());
                    }
                    let _ = tx.send(PartialMessage::done().message());
                });

                self.llm
                    .stream_llm_answer(machine_input, None, tx_p)
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

    async fn prepare_answer_data(
        &self,
        user_query: &str,
        citation_format: &CitationStyle,
    ) -> Result<(Vec<Source>, LlmInput), <Self as QueryEngine>::E> {
        let (document_indices, documents, formatted_documents) =
            self.get_documents(user_query).await?;

        let system = self
            .system_prompt
            .replace("###DOCUMENT_LIST###", &formatted_documents);

        let input = LlmInput {
            system,
            conversation: vec![LlmMessage {
                role: LlmRole::User,
                content: format!("Obey the instructions in the system prompt. You must cite every statement [1] and provide your answer in a long-form essay, formatted as markdown. Delimite the essay from the reference list with exactly the line '# Sources List:'. If you don't follow these instructions to the letter, my boss will fire me.\n{user_query}"),
            }],
        };

        Ok((
            documents
                .into_iter()
                .zip(document_indices)
                .map(|((ordinal, origin_text, provenance), index)| Source {
                    ordinal,
                    index,
                    citation: provenance.format(citation_format),
                    url: provenance.url(),
                    origin_text,
                })
                .collect(),
            input,
        ))
    }

    async fn get_documents(
        &self,
        user_query: &str,
    ) -> Result<(Vec<i64>, Vec<(usize, String, Provenance)>, String), <Self as QueryEngine>::E>
    {
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
                DocumentFormatter::format_document(*ordianal, &provenance.title(), document)
            })
            .collect::<Vec<String>>()
            .join("\n\n");
        Ok((document_indices, documents, formatted_documents))
    }
}
