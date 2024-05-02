use std::sync::Arc;

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
    },
    Client,
};
use futures::StreamExt;
use tera::Tera;
use tokio::sync::{mpsc::UnboundedSender, RwLock};

use super::{
    error::LlmClientError, LanguageServiceArguments, LlmClient, LlmClientBackend,
    LlmClientBackendKind, LlmMessage, LlmRole,
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

impl LlmClient<OpenAiInstructClient> {
    pub(crate) async fn new(
        client: OpenAiInstructClient,
        tera: Arc<RwLock<Tera>>,
    ) -> Result<Self, LlmClientError> {
        Ok(Self { client, tera })
    }
}

impl LlmClientBackendKind for OpenAiInstructClient {}

impl LlmClientBackend for LlmClient<OpenAiInstructClient> {
    async fn get_response<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<String, LlmClientError> {
        let prompt = arguments
            .messages
            .into_iter()
            .map(|LlmMessage { role, content }| match role {
                LlmRole::Assistant => {
                    let mut message = ChatCompletionRequestAssistantMessage::default();
                    message.content = Some(content);
                    ChatCompletionRequestMessage::Assistant(message)
                }
                LlmRole::User => {
                    let mut message = ChatCompletionRequestUserMessage::default();
                    message.content = ChatCompletionRequestUserMessageContent::Text(content);
                    ChatCompletionRequestMessage::User(message)
                }
                LlmRole::System => {
                    let mut message = ChatCompletionRequestSystemMessage::default();
                    message.content = content;
                    ChatCompletionRequestMessage::System(message)
                }
                LlmRole::Function => todo!(),
                LlmRole::Tool => todo!(),
            })
            .collect::<Vec<_>>();
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(max_tokens)
            .model(&self.client.model_name)
            .n(1)
            .messages(prompt)
            .stop(stop_phrases.iter().map(AsRef::as_ref).collect::<Vec<_>>())
            .build()?;

        let response = self.client.client.chat().create(request).await?;

        let response = response
            .choices
            .into_iter()
            .next()
            .ok_or(LlmClientError::EmptyResponse)?
            .message
            .content
            .ok_or(LlmClientError::EmptyResponse)?;
        Ok(response)
    }

    async fn stream_response<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments,
        tx: UnboundedSender<String>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<(), LlmClientError> {
        let prompt = arguments
            .messages
            .into_iter()
            .map(|LlmMessage { role, content }| match role {
                LlmRole::Assistant => {
                    let mut message = ChatCompletionRequestAssistantMessage::default();
                    message.content = Some(content);
                    ChatCompletionRequestMessage::Assistant(message)
                }
                LlmRole::User => {
                    let mut message = ChatCompletionRequestUserMessage::default();
                    message.content = ChatCompletionRequestUserMessageContent::Text(content);
                    ChatCompletionRequestMessage::User(message)
                }
                LlmRole::System => {
                    let mut message = ChatCompletionRequestSystemMessage::default();
                    message.content = content;
                    ChatCompletionRequestMessage::System(message)
                }
                LlmRole::Function => todo!(),
                LlmRole::Tool => todo!(),
            })
            .collect::<Vec<_>>();
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(max_tokens)
            .model(&self.client.model_name)
            .n(1)
            .messages(prompt)
            .stop(stop_phrases.iter().map(AsRef::as_ref).collect::<Vec<_>>())
            .build()?;

        let mut stream = self.client.client.chat().create_stream(request).await?;

        while let Some(Ok(fragment)) = stream.next().await {
            let response = fragment
                .choices
                .into_iter()
                .next()
                .ok_or(LlmClientError::EmptyResponse)?
                .delta
                .content
                .ok_or(LlmClientError::EmptyResponse)?;

            let _ = tx.send(response);
        }

        Ok(())
    }
}
