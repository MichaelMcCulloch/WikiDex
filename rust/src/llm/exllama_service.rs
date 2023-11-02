use reqwest::Client;
use url::Url;

use crate::config::LlmConfig;

use super::{protocol::LlmInput, LlmService, LlmServiceError};

pub(crate) struct ExLlamaExampleService {
    client: Client,
    host: Url,
}

#[async_trait::async_trait]
impl LlmService for ExLlamaExampleService {
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

impl ExLlamaExampleService {
    pub(crate) fn new(config: LlmConfig) -> Result<Self, url::ParseError> {
        let host: Url = config.into();
        let client = Client::new();

        Ok(Self { client, host })
    }
}
