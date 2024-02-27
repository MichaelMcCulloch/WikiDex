use super::{delegate::LanguageServiceArguments, error::LlmServiceError, protocol::LlmRole};
use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs, ListModelResponse,
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
    pub(crate) async fn up(&self) -> Result<ListModelResponse, OpenAIError> {
        self.chat_client.models().list().await
    }
    pub(super) fn new(chat_client: Client<OpenAIConfig>, chat_model_name: String) -> Self {
        ChatClient {
            chat_client,
            chat_model_name,
        }
    }

    pub(crate) async fn get_response(
        &self,
        arguments: LanguageServiceArguments<'_>,
        stop_phrases: Vec<&str>,
    ) -> Result<String, LlmServiceError> {
        let request = self.create_chat_request(arguments, stop_phrases)?;
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
        arguments: LanguageServiceArguments<'_>,
        tx: UnboundedSender<String>,
        stop_phrases: Vec<&str>,
    ) -> Result<(), LlmServiceError> {
        let request = self.create_chat_request(arguments, stop_phrases)?;

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
        arguments: LanguageServiceArguments,
        stop_phrases: Vec<&str>,
    ) -> Result<CreateChatCompletionRequest, LlmServiceError>;
}
impl ChatRequest for ChatClient {
    fn create_chat_request(
        &self,
        arguments: LanguageServiceArguments,
        stop_phrases: Vec<&str>,
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
            .stop(stop_phrases)
            .build()
            .map_err(LlmServiceError::AsyncOpenAiError)?;

        Ok(request)
    }
}
