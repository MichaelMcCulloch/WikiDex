pub(crate) mod protocol;
pub(crate) mod vllm;

use self::protocol::{LlmInput, LlmRole};
use std::fmt::{self, Display, Formatter};

#[async_trait::async_trait]
pub(crate) trait LlmService {
    type E;
    async fn get_llm_answer(&self, input: LlmInput) -> Result<LlmInput, Self::E>;
}

#[derive(Debug)]
pub(crate) enum LlmServiceError {
    OpenAIError(async_openai::error::OpenAIError),
    EmptyResponse,
    UnexpectedRole(LlmRole),
}

impl std::error::Error for LlmServiceError {}

impl Display for LlmServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LlmServiceError::OpenAIError(err) => write!(f, "LLMService: {}", err),
            LlmServiceError::EmptyResponse => write!(f, "LLMService: Empty Response from service"),
            LlmServiceError::UnexpectedRole(r) => {
                write!(f, "LLMService: Unexpected role '{r}' from service.")
            }
        }
    }
}
