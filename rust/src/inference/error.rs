use std::fmt::{Display, Formatter, Result};

use crate::{
    docstore::DocstoreRetrieveError,
    index::IndexSearchError,
    openai::{EmbeddingServiceError, LlmServiceError},
};

#[derive(Debug)]
pub(crate) enum QueryEngineError {
    DocstoreError(DocstoreRetrieveError),
    LlmServiceError(LlmServiceError),
    EmbeddingServiceError(EmbeddingServiceError),
    EmptyConversation,
    IndexError(IndexSearchError),
    IndexOutOfRange,
    InvalidAgentResponse,
    LastMessageIsNotUser,
    LlmError(LlmServiceError),
    UnableToLockIndex,
}

impl std::error::Error for QueryEngineError {}

impl Display for QueryEngineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            QueryEngineError::DocstoreError(err) => {
                write!(f, "{}", err)
            }
            QueryEngineError::LlmServiceError(err) => {
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
            QueryEngineError::IndexOutOfRange => write!(f, "QueryEngine: Index out of range error"),
            QueryEngineError::InvalidAgentResponse => {
                write!(f, "QueryEngine: Invalid agent response error")
            }
            QueryEngineError::LastMessageIsNotUser => {
                write!(f, "QueryEngine: Last message is not from a user error")
            }
            QueryEngineError::UnableToLockIndex => {
                write!(f, "QueryEngine: Unable to lock index error")
            }
        }
    }
}
