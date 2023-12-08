use crate::llm::protocol::PartialLlmMessage;

use super::{AsyncLlmService, LlmInput, LlmMessage, LlmRole, LlmServiceError};
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
    Client,
};
use backoff::future::retry;
use backoff::ExponentialBackoff;
use futures::StreamExt;
use url::Url;

use tokio::sync::mpsc::UnboundedSender;
pub(crate) struct AsyncOpenAiService {
    client: Client<OpenAIConfig>,
    model_name: String,
    pub(crate) model_context_length: usize,
}

#[async_trait::async_trait]
impl AsyncLlmService for AsyncOpenAiService {
    type E = LlmServiceError;
    async fn get_llm_answer(
        &self,
        input: LlmInput,
        max_new_tokens: Option<u16>,
    ) -> Result<LlmMessage, Self::E> {
        let message_openai_compat: Result<
            Vec<ChatCompletionRequestMessage>,
            <AsyncOpenAiService as AsyncLlmService>::E,
        > = input.into();

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(max_new_tokens.unwrap_or(2048u16))
            .model(self.model_name.clone())
            .messages(message_openai_compat?)
            .build()
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        let response = response
            .choices
            .into_iter()
            .next()
            .ok_or(LlmServiceError::EmptyResponse)?;

        match (
            LlmRole::from(&response.message.role),
            response.message.content,
        ) {
            (LlmRole::System, _) => Err(LlmServiceError::UnexpectedRole(LlmRole::System)),
            (LlmRole::Function, _) => Err(LlmServiceError::UnexpectedRole(LlmRole::Function)),
            (_, None) => Err(LlmServiceError::EmptyResponse),
            (role, Some(content)) => Ok(LlmMessage { role, content }),
        }
    }
    async fn stream_llm_answer(
        &self,
        input: LlmInput,
        max_new_tokens: Option<u16>,
        tx: UnboundedSender<PartialLlmMessage>,
    ) -> Result<(), Self::E> {
        let message_openai_compat: Result<
            Vec<ChatCompletionRequestMessage>,
            <AsyncOpenAiService as AsyncLlmService>::E,
        > = input.into();

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(max_new_tokens.unwrap_or(2048u16))
            .model(self.model_name.clone())
            .messages(message_openai_compat?)
            .build()
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        let mut stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        let _ = stream.next().await; //Skip the first element
        while let Some(Ok(fragment)) = stream.next().await {
            let response = fragment
                .choices
                .into_iter()
                .next()
                .ok_or(LlmServiceError::EmptyResponse)?;
            if let Some(role) = response.delta.role {
                tx.send(PartialLlmMessage {
                    role: Some(LlmRole::from(&role)),
                    content: None,
                })
                .unwrap();
            }
            if let Some(content) = response.delta.content {
                let _ = tx.send(PartialLlmMessage {
                    role: None,
                    content: Some(content),
                });
            }
        }

        Ok(())
    }
    async fn wait_for_service(&self) -> Result<(), LlmServiceError> {
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1u16)
            .model(self.model_name.clone())
            .messages(vec![])
            .build()
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        retry(ExponentialBackoff::default(), || async {
            Ok(self.client.chat().create(request.clone()).await?)
        })
        .await
        .map_err(LlmServiceError::AsyncOpenAiError)?;

        Ok(())
    }
}

impl AsyncOpenAiService {
    pub(crate) fn new<S: AsRef<str>>(
        host: Url,
        model_name: S,
        model_context_length: usize,
    ) -> Self {
        let openai_config = OpenAIConfig::new().with_api_base(host);
        let client = Client::with_config(openai_config);
        let model_name = model_name.as_ref().to_string();
        Self {
            client,
            model_name,
            model_context_length,
        }
    }
}
