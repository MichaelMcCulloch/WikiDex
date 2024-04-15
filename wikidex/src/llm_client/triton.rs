use tokio::sync::mpsc::UnboundedSender;

use crate::openai::LanguageServiceArguments;

use super::{error::LlmClientError, LlmClient, LlmClientService, TritonClient};

impl LlmClient<TritonClient> {
    pub(crate) async fn new<S: AsRef<str>>(triton_url: S) -> Result<Self, LlmClientError> {
        let client = TritonClient::new(triton_url.as_ref(), None)
            .await
            .map_err(LlmClientError::TritonClient)?;
        Ok(Self { client })
    }
}
#[cfg(feature = "triton")]
impl LlmClientService for TritonClient {
    async fn get_response<S: AsRef<str>>(
        &self,
        _arguments: LanguageServiceArguments<'_>,
        _max_tokens: u16,
        _stop_phrases: Vec<S>,
    ) -> Result<String, LlmClientError> {
        todo!()
    }

    async fn stream_response<S: AsRef<str>>(
        &self,
        _arguments: LanguageServiceArguments<'_>,
        _tx: UnboundedSender<String>,
        _max_tokens: u16,
        _stop_phrases: Vec<S>,
    ) -> Result<(), LlmClientError> {
        todo!()
    }
}

#[cfg(feature = "triton")]
impl LlmClientService for LlmClient<TritonClient> {
    async fn get_response<S: AsRef<str>>(
        &self,
        _arguments: LanguageServiceArguments<'_>,
        _max_tokens: u16,
        _stop_phrases: Vec<S>,
    ) -> Result<String, LlmClientError> {
        todo!()
    }

    async fn stream_response<S: AsRef<str>>(
        &self,
        _arguments: LanguageServiceArguments<'_>,
        _tx: UnboundedSender<String>,
        _max_tokens: u16,
        _stop_phrases: Vec<S>,
    ) -> Result<(), LlmClientError> {
        todo!()
    }
}
