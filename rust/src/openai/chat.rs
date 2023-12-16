use super::{
    delegate::LanguageServiceServiceArguments,
    error::LlmServiceError,
    protocol::{LlmMessage, LlmRole, PartialLlmMessage},
    service::TCompletionClient,
};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use futures::StreamExt;
use std::error::Error;
use tokio::sync::mpsc::UnboundedSender;

#[async_trait::async_trait]
pub(crate) trait ChatService {
    type E: Error;
    async fn answer(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
    ) -> Result<LlmMessage, Self::E>;
    async fn stream_answer(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
        tx: UnboundedSender<PartialLlmMessage>,
    ) -> Result<(), Self::E>;
}

pub(crate) struct ChatCompletionClient {
    chat_client: Client<OpenAIConfig>,
    chat_model_name: String,
}

impl ChatCompletionClient {
    pub(super) fn new(chat_client: Client<OpenAIConfig>, chat_model_name: String) -> Self {
        ChatCompletionClient {
            chat_client,
            chat_model_name,
        }
    }
}

pub(crate) trait TChat {
    fn create_chat_request(
        &self,
        arguments: LanguageServiceServiceArguments,
    ) -> Result<CreateChatCompletionRequest, LlmServiceError>;
}
#[async_trait::async_trait]
impl TCompletionClient for ChatCompletionClient {
    async fn get_response(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
    ) -> Result<String, LlmServiceError> {
        let request = self.create_chat_request(arguments)?;
        let response = self
            .chat_client
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
            (role, Some(content)) => Ok(content),
        }
    }

    async fn stream_response(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
        tx: UnboundedSender<String>,
    ) -> Result<(), LlmServiceError> {
        let request = self.create_chat_request(arguments)?;

        let mut stream = self
            .chat_client
            .chat()
            .create_stream(request)
            .await
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        let _ = stream.next().await;
        while let Some(Ok(fragment)) = stream.next().await {
            let response = fragment
                .choices
                .into_iter()
                .next()
                .ok_or(LlmServiceError::EmptyResponse)?;
            if let Some(role) = response.delta.role {}
            if let Some(content) = response.delta.content {
                let _ = tx.send(content);
            }
        }

        Ok(())
    }
}
const PROMPT_SALT: &str = "";
impl TChat for ChatCompletionClient {
    fn create_chat_request(
        &self,
        arguments: LanguageServiceServiceArguments,
    ) -> Result<CreateChatCompletionRequest, LlmServiceError> {
        let query = format!("{PROMPT_SALT}\n{}", arguments.query);

        let system = arguments
            .system
            .replace("___DOCUMENT_LIST___", &arguments.documents);

        let system = ChatCompletionRequestSystemMessageArgs::default()
            .content(system)
            .build()
            .map(|e| ChatCompletionRequestMessage::System(e))
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        let query = ChatCompletionRequestUserMessageArgs::default()
            .content(query)
            .build()
            .map(|e| ChatCompletionRequestMessage::User(e))
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        let message_openai_compat = vec![system, query];

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(2048u16)
            .model(self.chat_model_name.clone())
            .messages(message_openai_compat)
            .stop("References:")
            .build()
            .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        Ok(request)
    }
}
