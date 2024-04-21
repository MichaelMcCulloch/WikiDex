use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

use crate::{embedding_client::EmbeddingServiceError, llm_client::LlmClientError};

#[derive(Debug)]
pub(crate) enum WikiMarkupProcessingError {
    Llm(LlmClientError),
    Embed(EmbeddingServiceError),
}

impl Error for WikiMarkupProcessingError {}
impl Display for WikiMarkupProcessingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            WikiMarkupProcessingError::Llm(e) => {
                write!(f, "{e}")
            }
            WikiMarkupProcessingError::Embed(e) => {
                write!(f, "{e}")
            }
        }
    }
}
