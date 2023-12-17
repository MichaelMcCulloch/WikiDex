use super::{delegate::LanguageServiceServiceArguments, error::LlmServiceError, protocol::LlmRole};
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
use tokio::sync::mpsc::UnboundedSender;
const PROMPT_SALT: &str = "";

pub(crate) struct ChatClient {
    chat_client: Client<OpenAIConfig>,
    chat_model_name: String,
}

impl ChatClient {
    pub(super) fn new(chat_client: Client<OpenAIConfig>, chat_model_name: String) -> Self {
        ChatClient {
            chat_client,
            chat_model_name,
        }
    }
}

impl ChatClient {
    pub(crate) async fn get_response(
        &self,
        arguments: LanguageServiceServiceArguments<'_>,
    ) -> Result<String, LlmServiceError> {
        let request = self.create_chat_request(arguments)?;
        let response = self
            .chat_client
            .chat()
            .create(request)
            .await
            .map_err(LlmServiceError::AsyncOpenAiError)?;

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
            (_role, Some(content)) => Ok(content),
        }
    }

    pub(crate) async fn stream_response(
        &self,
        arguments: LanguageServiceServiceArguments<'_>,
        tx: UnboundedSender<String>,
    ) -> Result<(), LlmServiceError> {
        let request = self.create_chat_request(arguments)?;

        let mut stream = self
            .chat_client
            .chat()
            .create_stream(request)
            .await
            .map_err(LlmServiceError::AsyncOpenAiError)?;

        let _ = stream.next().await;
        while let Some(Ok(fragment)) = stream.next().await {
            let response = fragment
                .choices
                .into_iter()
                .next()
                .ok_or(LlmServiceError::EmptyResponse)?;
            if let Some(_role) = response.delta.role {}
            if let Some(content) = response.delta.content {
                let _ = tx.send(content);
            }
        }

        Ok(())
    }
}
pub(crate) trait ChatRequest {
    fn create_chat_request(
        &self,
        arguments: LanguageServiceServiceArguments,
    ) -> Result<CreateChatCompletionRequest, LlmServiceError>;
}
impl ChatRequest for ChatClient {
    fn create_chat_request(
        &self,
        arguments: LanguageServiceServiceArguments,
    ) -> Result<CreateChatCompletionRequest, LlmServiceError> {
        let query = format!("{PROMPT_SALT}\n{}", arguments.query);

        let system = arguments
            .system
            .replace("___DOCUMENT_LIST___", arguments.documents);

        let system = ChatCompletionRequestSystemMessageArgs::default()
            .content(system)
            .build()
            .map(ChatCompletionRequestMessage::System)
            .map_err(LlmServiceError::AsyncOpenAiError)?;

        let query = ChatCompletionRequestUserMessageArgs::default()
            .content(query)
            .build()
            .map(ChatCompletionRequestMessage::User)
            .map_err(LlmServiceError::AsyncOpenAiError)?;

        let message_openai_compat = vec![system, query];

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(2048u16)
            .model(self.chat_model_name.clone())
            .messages(message_openai_compat)
            .stop("References:")
            .build()
            .map_err(LlmServiceError::AsyncOpenAiError)?;

        Ok(request)
    }
}
