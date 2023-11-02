pub(crate) mod protocol;

use self::protocol::LlmInput;
use std::fmt::{self, Display, Formatter};

pub(crate) mod exllama_service;
pub(crate) mod vllm;

#[async_trait::async_trait]
pub(crate) trait LlmService {
    type E;
    async fn get_llm_answer(&self, input: LlmInput) -> Result<LlmInput, Self::E>;
}

#[derive(Debug)]
pub(crate) enum LlmServiceError {
    ReqwestError(reqwest::Error),
    OpenAIError(async_openai::error::OpenAIError),
    EmptyResponse,
    SerializationError,
    UnexpectedResponse,
}

impl std::error::Error for LlmServiceError {}

impl Display for LlmServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LlmServiceError::ReqwestError(err) => write!(f, "LLMService: {}", err),
            LlmServiceError::OpenAIError(err) => write!(f, "LLMService: {}", err),
            LlmServiceError::SerializationError => write!(f, "LLMService: Serialization error"),
            LlmServiceError::EmptyResponse => write!(f, "LLMService: Empty Response from service"),
            LlmServiceError::UnexpectedResponse => {
                write!(f, "LLMService: Unexpected Response from service")
            }
        }
    }
}
