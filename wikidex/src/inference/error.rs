use crate::llm_client::LlmClientError;
use crate::{
    docstore::DocstoreRetrieveError, embedding_client::EmbeddingServiceError,
    index::IndexSearchError,
};
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum QueryEngineError {
    DocstoreError(DocstoreRetrieveError),
    EmbeddingServiceError(EmbeddingServiceError),
    EmptyConversation,
    IndexError(IndexSearchError),
    InvalidAgentResponse,
    LastMessageIsNotUser,
    LlmError(LlmClientError),
    Tera(tera::Error),
}

impl From<tera::Error> for QueryEngineError {
    fn from(value: tera::Error) -> Self {
        Self::Tera(value)
    }
}

impl From<DocstoreRetrieveError> for QueryEngineError {
    fn from(value: DocstoreRetrieveError) -> Self {
        Self::DocstoreError(value)
    }
}
impl From<EmbeddingServiceError> for QueryEngineError {
    fn from(value: EmbeddingServiceError) -> Self {
        Self::EmbeddingServiceError(value)
    }
}
impl From<IndexSearchError> for QueryEngineError {
    fn from(value: IndexSearchError) -> Self {
        Self::IndexError(value)
    }
}
impl From<LlmClientError> for QueryEngineError {
    fn from(value: LlmClientError) -> Self {
        Self::LlmError(value)
    }
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
            QueryEngineError::Tera(err) => write!(f, "{}", err),
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
