use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum EmbeddingServiceError {
    AsyncOpenAiError(async_openai::error::OpenAIError),
    EmbeddingSizeMismatch(usize, usize),
}

impl From<async_openai::error::OpenAIError> for EmbeddingServiceError {
    fn from(value: async_openai::error::OpenAIError) -> Self {
        Self::AsyncOpenAiError(value)
    }
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
