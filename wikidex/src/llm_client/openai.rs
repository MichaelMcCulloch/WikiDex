use async_openai::{config::OpenAIConfig, types::CreateCompletionRequestArgs, Client};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;

use super::{
    error::LlmClientError, LanguageServiceArguments, LlmClient, LlmClientBackend,
    LlmClientBackendKind, LlmClientService,
};

pub(crate) struct OpenAiInstructClient {
    client: Client<OpenAIConfig>,
    model_name: String,
}

impl OpenAiInstructClient {
    pub(crate) fn new(client: Client<OpenAIConfig>, model_name: String) -> Self {
        Self { client, model_name }
    }
}

pub(crate) struct OpenAiChatClient {
    client: Client<OpenAIConfig>,
    model_name: String,
}

impl OpenAiChatClient {
    pub(crate) fn new(client: Client<OpenAIConfig>, model_name: String) -> Self {
        Self { client, model_name }
    }
}

impl LlmClient<OpenAiInstructClient> {
    pub(crate) async fn new<O: AsRef<str>, M: AsRef<str>>(
        openai_url: O,
        model_name: M,
    ) -> Result<Self, LlmClientError> {
        let openai_config = OpenAIConfig::new().with_api_base(openai_url.as_ref());
        let open_ai_client = Client::with_config(openai_config);
        let client = OpenAiInstructClient::new(open_ai_client, model_name.as_ref().to_string());
        Ok(Self { client })
    }
}

impl LlmClientBackendKind for OpenAiInstructClient {}

impl LlmClientBackend for LlmClient<OpenAiInstructClient> {
    async fn get_response<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments<'_>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<String, LlmClientError> {
        let prompt = self.fill_rag_template(arguments);
        let request = CreateCompletionRequestArgs::default()
            .max_tokens(max_tokens)
            .model(&self.client.model_name)
            .n(1)
            .prompt(prompt)
            .stop(stop_phrases.iter().map(AsRef::as_ref).collect::<Vec<_>>())
            .build()
            .map_err(LlmClientError::OpenAiClient)?;

        let response = self
            .client
            .client
            .completions()
            .create(request)
            .await
            .map_err(LlmClientError::OpenAiClient)?;

        let response = response
            .choices
            .into_iter()
            .next()
            .ok_or(LlmClientError::EmptyResponse)?;
        Ok(response.text)
    }

    async fn stream_response<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments<'_>,
        tx: UnboundedSender<String>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<(), LlmClientError> {
        let prompt = self.fill_rag_template(arguments);
        let request = CreateCompletionRequestArgs::default()
            .max_tokens(max_tokens)
            .model(&self.client.model_name)
            .n(1)
            .prompt(prompt)
            .stop(stop_phrases.iter().map(AsRef::as_ref).collect::<Vec<_>>())
            .build()
            .map_err(LlmClientError::OpenAiClient)?;

        let mut stream = self
            .client
            .client
            .completions()
            .create_stream(request)
            .await
            .map_err(LlmClientError::OpenAiClient)?;

        while let Some(Ok(fragment)) = stream.next().await {
            let response = fragment
                .choices
                .into_iter()
                .next()
                .ok_or(LlmClientError::EmptyResponse)?;

            let _ = tx.send(response.text);
        }

        Ok(())
    }
}