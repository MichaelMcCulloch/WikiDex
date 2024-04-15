use std::fmt::{Display, Formatter, Result};

use crate::{
    docstore::DocstoreRetrieveError, embedding_client::EmbeddingServiceError,
    index::IndexSearchError, llm_client::LlmClientError,
};

#[derive(Debug)]
pub(crate) enum QueryEngineError {
    DocstoreError(DocstoreRetrieveError),
    EmbeddingServiceError(EmbeddingServiceError),
    EmptyConversation,
    IndexError(IndexSearchError),
    InvalidAgentResponse,
    LastMessageIsNotUser,
    LlmError(LlmClientError),
}

impl std::error::Error for QueryEngineError {}

impl Display for QueryEngineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            QueryEngineError::DocstoreError(err) => {
                write!(f, "{}", err)
            }

            QueryEngineError::EmbeddingServiceError(err) => {
                write!(f, "{}", err)
            }
            QueryEngineError::IndexError(err) => write!(f, "{}", err),
            QueryEngineError::LlmError(err) => write!(f, "{}", err),
            QueryEngineError::EmptyConversation => {
                write!(f, "QueryEngine: Empty conversation error")
            }
            QueryEngineError::InvalidAgentResponse => {
                write!(f, "QueryEngine: Invalid agent response error")
            }
            QueryEngineError::LastMessageIsNotUser => {
                write!(f, "QueryEngine: Last message is not from a user error")
            }
        }
    }
}
