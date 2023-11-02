pub(crate) mod protocol;

use self::protocol::LlmInput;
use crate::config::LlmConfig;
use reqwest::Client;
use std::fmt::{self, Display, Formatter};
use url::Url;

pub(crate) mod exllama_service;

#[async_trait::async_trait]
pub(crate) trait LlmService {
    type E;
    async fn get_llm_answer(&self, input: LlmInput) -> Result<LlmInput, Self::E>;
}

#[derive(Debug)]
pub(crate) enum LlmServiceError {
    ReqwestError(reqwest::Error),
    SerializationError,
}

impl std::error::Error for LlmServiceError {}

impl Display for LlmServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LlmServiceError::ReqwestError(err) => write!(f, "LLMService: {}", err),
            LlmServiceError::SerializationError => write!(f, "LLMService: Serialization error"),
        }
    }
}
