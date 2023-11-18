mod client;
mod error;

use async_openai::config::OpenAIConfig;
pub(crate) use error::OpenAiClientError;

use backoff::{retry, Error, ExponentialBackoff};
use client::{OpenAIClient, SyncOpenAiClient};
use url::Url;

use super::super::{LlmInput, LlmServiceError, SyncLlmService};

pub(crate) struct SyncOpenAiService {
    client: SyncOpenAiClient,
    model_name: String,
    pub(crate) model_context_length: usize,
}

impl SyncLlmService for SyncOpenAiService {
    type E = LlmServiceError;
    fn get_llm_answer(
        &self,
        input: LlmInput,
        max_new_tokens: Option<u16>,
    ) -> Result<LlmInput, Self::E> {
        Ok(input)
    }
    fn wait_for_service(&self) -> Result<(), LlmServiceError> {
        let op = || self.client.test(&self.model_name).map_err(Error::transient);

        let _ = retry(ExponentialBackoff::default(), op);

        Ok(())
    }
}

impl SyncOpenAiService {
    pub(crate) fn new<S: AsRef<str>>(
        host: Url,
        model_name: S,
        model_context_length: usize,
    ) -> Self {
        let openai_config = OpenAIConfig::new().with_api_base(host);
        let client = SyncOpenAiClient::with_config(openai_config);
        let model_name = model_name.as_ref().to_string();
        Self {
            client,
            model_name,
            model_context_length,
        }
    }
}
