use std::fmt::{Display, Formatter, Result};

use super::LlmRole;

#[derive(Debug)]
pub(crate) enum LlmServiceError {
    OpenAIError(async_openai::error::OpenAIError),
    EmptyResponse,
    UnexpectedRole(LlmRole),
}

impl std::error::Error for LlmServiceError {}

impl Display for LlmServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            LlmServiceError::OpenAIError(err) => write!(f, "LLMService: {}", err),
            LlmServiceError::EmptyResponse => write!(f, "LLMService: Empty Response from service"),
            LlmServiceError::UnexpectedRole(r) => {
                write!(f, "LLMService: Unexpected role '{r}' from service.")
            }
        }
    }
}
