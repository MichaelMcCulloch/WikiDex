use std::sync::Mutex;

use crate::{
    docstore::{DocumentService, SqliteDocstore},
    embed::{r#async::Embedder, EmbedService},
    formatter::{DocumentFormatter, TextFormatter},
    index::{FaissIndex, SearchService},
    llm::{
        AsyncLlmService, AsyncOpenAiService, {LlmInput, LlmMessage, LlmRole},
    },
    server::{Conversation, Message},
};

use super::{QueryEngine, QueryEngineError};

pub struct Engine {
    index: Mutex<FaissIndex>,
    embed: Embedder,
    docstore: SqliteDocstore,
    llm: AsyncOpenAiService,
    prompt: String,
}

#[async_trait::async_trait]
impl QueryEngine for Engine {
    type E = QueryEngineError;
    async fn query(&self, question: &str) -> Result<String, Self::E> {
        let embedding = self
            .embed
            .embed(&[&question])
            .await
            .map_err(|e| QueryEngineError::EmbeddingError(e))?;

        let first_embedding = embedding
            .iter()
            .next()
            .ok_or(QueryEngineError::IndexOutOfRange)?;

        let result = self
            .index
            .lock()
            .map_err(|_| QueryEngineError::UnableToLockIndex)?
            .search(first_embedding, 8)
            .map_err(|e| QueryEngineError::IndexError(e))?;

        let documents = self
            .docstore
            .retreive(&result)
            .await
            .map_err(|e| QueryEngineError::DocstoreError(e))?;

        let response = documents
            .iter()
            .map(|(index, document, _)| DocumentFormatter::format_document(*index, document))
            .collect::<Vec<String>>()
            .join("\n\n");

        Ok(response)
    }

    async fn conversation(
        &self,
        Conversation(message_history): &Conversation,
    ) -> Result<Message, Self::E> {
        match message_history.last() {
            Some(Message::User(user_query)) => {
                let embedding = self
                    .embed
                    .embed(&[user_query])
                    .await
                    .map_err(|e| QueryEngineError::EmbeddingError(e))?;

                let first_embedding = embedding
                    .iter()
                    .next()
                    .ok_or(QueryEngineError::IndexOutOfRange)?;

                let document_indices = self
                    .index
                    .lock()
                    .map_err(|_| QueryEngineError::UnableToLockIndex)?
                    .search(first_embedding, 8)
                    .map_err(|e| QueryEngineError::IndexError(e))?;

                let documents = self
                    .docstore
                    .retreive(&document_indices)
                    .await
                    .map_err(|e| QueryEngineError::DocstoreError(e))?;

                let formatted_document_list = documents
                    .iter()
                    .map(|(index, document, _provenance)| {
                        DocumentFormatter::format_document(*index, document)
                    })
                    .collect::<Vec<String>>()
                    .join("\n\n");
                let system = self
                    .prompt
                    .replace("###DOCUMENT_LIST###", &formatted_document_list)
                    .replace("###USER_QUERY###", user_query);

                let input = LlmInput {
                    system,
                    conversation: vec![LlmMessage {
                        role: LlmRole::User,
                        content: format!("{}", user_query),
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
                            .iter()
                            .map(|(i, d, _)| (format!("{i}"), format!("{d}")))
                            .collect(),
                    )),
                    _ => Err(QueryEngineError::InvalidAgentResponse)?,
                }
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
