mod client;
mod error;

use async_openai::config::OpenAIConfig;
pub(crate) use error::SynchronousOpenAiClientError;

use backoff::{retry, Error, ExponentialBackoff};
use client::{OpenAIClient, SyncOpenAiClient};
use url::Url;

use crate::llm::{protocol::OpenAiJsonFormat, LlmMessage, LlmRole};

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
    ) -> Result<LlmMessage, Self::E> {
        let compat: Result<OpenAiJsonFormat, <SyncOpenAiService as SyncLlmService>::E> =
            input.into();
        let response = self
            .client
            .get_completion_for_conversation(
                compat?,
                &self.model_name,
                max_new_tokens.unwrap_or(2048u16),
            )
            .map_err(LlmServiceError::SyncOpenAiError)?;

        Ok(LlmMessage {
            role: LlmRole::Assistant,
            content: response
                .choices
                .into_iter()
                .next()
                .ok_or(LlmServiceError::EmptyResponse)?
                .text,
        })
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
