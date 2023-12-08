use std::fmt::{Display, Formatter, Result};

use super::{openai, LlmRole};

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
