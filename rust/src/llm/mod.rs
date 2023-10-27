pub(crate) mod protocol;

use crate::config::LlmConfig;
use reqwest::Client;
use std::fmt::{self, Display, Formatter};
use url::Url;

use self::protocol::LlmInput;
pub(crate) struct Llm {
    client: Client,
    host: Url,
}
#[async_trait::async_trait]
pub(crate) trait LlmService {
    type E;
    async fn get_llm_answer(&self, input: LlmInput) -> Result<LlmInput, Self::E>;
}

#[async_trait::async_trait]
impl LlmService for Llm {
    type E = LlmServiceError;
    async fn get_llm_answer(&self, input: LlmInput) -> Result<LlmInput, Self::E> {
        let request_body =
            serde_json::to_string(&input).map_err(|_| LlmServiceError::SerializationError)?;
        let response: LlmInput = self
            .client
            .post(self.host.clone())
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LlmServiceError::ReqwestError(e))?
            .json()
            .await
            .map_err(|e| LlmServiceError::ReqwestError(e))?;
        Ok(response)
    }
}

impl Llm {
    pub(crate) fn new(config: LlmConfig) -> Result<Self, url::ParseError> {
        let host: Url = config.into();
        let client = Client::new();

        Ok(Self { client, host })
    }
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
