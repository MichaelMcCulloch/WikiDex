use std::fmt::{Display, Formatter, Result};

use crate::{
    docstore::DocstoreRetrieveError,
    index::IndexSearchError,
    openai::{EmbeddingServiceError, LlmServiceError},
};

#[derive(Debug)]
pub(crate) enum PromptBreedingError {
    DocstoreError(DocstoreRetrieveError),
    EmbeddingServiceError(EmbeddingServiceError),
    EmptyConversation,
    IndexError(IndexSearchError),
    InvalidAgentResponse,
    LastMessageIsNotUser,
    LlmError(LlmServiceError),
    UnableToLockIndex,
}

impl std::error::Error for PromptBreedingError {}

impl Display for PromptBreedingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PromptBreedingError::DocstoreError(err) => {
                write!(f, "{}", err)
            }

            PromptBreedingError::EmbeddingServiceError(err) => {
                write!(f, "{}", err)
            }
            PromptBreedingError::IndexError(err) => write!(f, "{}", err),
            PromptBreedingError::LlmError(err) => write!(f, "{}", err),
            PromptBreedingError::EmptyConversation => {
                write!(f, "QueryEngine: Empty conversation error")
            }
            PromptBreedingError::InvalidAgentResponse => {
                write!(f, "QueryEngine: Invalid agent response error")
            }
            PromptBreedingError::LastMessageIsNotUser => {
                write!(f, "QueryEngine: Last message is not from a user error")
            }
            PromptBreedingError::UnableToLockIndex => {
                write!(f, "QueryEngine: Unable to lock index error")
            }
        }
    }
}
