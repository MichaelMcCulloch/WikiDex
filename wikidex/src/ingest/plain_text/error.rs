use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

use crate::{embedding_client::EmbeddingServiceError, llm_client::LlmClientError};

#[derive(Debug)]
pub(crate) enum PlainTextProcessingError {
    Llm(LlmClientError),
    Embed(EmbeddingServiceError),
}

impl From<LlmClientError> for PlainTextProcessingError {
    fn from(value: LlmClientError) -> Self {
        Self::Llm(value)
    }
}
impl From<EmbeddingServiceError> for PlainTextProcessingError {
    fn from(value: EmbeddingServiceError) -> Self {
        Self::Embed(value)
    }
}

impl Error for PlainTextProcessingError {}
impl Display for PlainTextProcessingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PlainTextProcessingError::Llm(e) => write!(f, "{:?}", e),
            PlainTextProcessingError::Embed(e) => write!(f, "{:?}", e),
        }
    }
}
