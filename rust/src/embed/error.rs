use reqwest::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum EmbeddingServiceError {
    Reqwuest(Error),
    EmbeddingSizeMismatch(usize, usize),
    AsyncOpenAiError(async_openai::error::OpenAIError),
}

impl std::error::Error for EmbeddingServiceError {}

impl Display for EmbeddingServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            EmbeddingServiceError::AsyncOpenAiError(err) => write!(f, "LLMService: {}", err),
            EmbeddingServiceError::Reqwuest(err) => {
                write!(f, "EmbeddingService: {}", err)
            }
            EmbeddingServiceError::EmbeddingSizeMismatch(expected, actual) => write!(
                f,
                "EmbeddingService: Embedding size mismatch. Expected: {}, Actual: {}",
                expected, actual
            ),
        }
    }
}
