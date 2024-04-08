use std::fmt::{Display, Formatter, Result};

use super::protocol::LlmRole;

#[derive(Debug)]
pub(crate) enum LlmServiceError {
    AsyncOpenAiError(async_openai::error::OpenAIError),
    EmptyResponse,
    UnexpectedRole(LlmRole),
}

impl std::error::Error for LlmServiceError {}

impl Display for LlmServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            LlmServiceError::AsyncOpenAiError(err) => write!(f, "LLMService: {}", err),
            LlmServiceError::EmptyResponse => write!(f, "LLMService: Empty Response from service"),
            LlmServiceError::UnexpectedRole(r) => {
                write!(f, "LLMService: Unexpected role '{r}' from service.")
            }
        }
    }
}
#[derive(Debug)]
pub(crate) enum EmbeddingServiceError {
    AsyncOpenAiError(async_openai::error::OpenAIError),
    EmbeddingSizeMismatch(usize, usize),
}

impl std::error::Error for EmbeddingServiceError {}

impl Display for EmbeddingServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            EmbeddingServiceError::AsyncOpenAiError(err) => write!(f, "LLMService: {}", err),
            EmbeddingServiceError::EmbeddingSizeMismatch(expected, actual) => write!(
                f,
                "EmbeddingService: Embedding size mismatch. Expected: {}, Actual: {}",
                expected, actual
            ),
        }
    }
}
