use super::super::{AsyncLlmService, LlmInput, LlmMessage, LlmRole, LlmServiceError};
use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use backoff::future::retry;
use backoff::ExponentialBackoff;
use url::Url;
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
    ) -> Result<LlmInput, Self::E> {
        let LlmInput {
            system,
            mut conversation,
        } = input;

        let system_openai_compat =
            role_message_to_request_message(&LlmRole::System, system.as_str())
                .map_err(|e| LlmServiceError::OpenAIError(e))?;

        let mut message_openai_compat = vec![system_openai_compat];

        for message in conversation.iter() {
            let LlmMessage { role, message } = message;
            let msg: ChatCompletionRequestMessage = role_message_to_request_message(&role, message)
                .map_err(|e| LlmServiceError::OpenAIError(e))?;
            message_openai_compat.push(msg);
        }

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(max_new_tokens.unwrap_or(2048u16))
            .model(self.model_name.clone())
            .messages(message_openai_compat)
            .build()
            .map_err(|e| LlmServiceError::OpenAIError(e))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| LlmServiceError::OpenAIError(e))?;

        let response = response
            .choices
            .into_iter()
            .next()
            .ok_or(LlmServiceError::EmptyResponse)?;

        let response = match (
            LlmRole::from(&response.message.role),
            response.message.content,
        ) {
            (LlmRole::System, _) => Err(LlmServiceError::UnexpectedRole(LlmRole::System)),
            (LlmRole::Function, _) => Err(LlmServiceError::UnexpectedRole(LlmRole::Function)),
            (_, None) => Err(LlmServiceError::EmptyResponse),
            (role, Some(message)) => Ok(LlmMessage { role, message }),
        }?;

        conversation.push(response);

        Ok(LlmInput {
            system,
            conversation,
        })
    }

    async fn wait_for_service(&self) -> Result<(), LlmServiceError> {
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1u16)
            .model(self.model_name.clone())
            .messages(vec![])
            .build()
            .map_err(|e| LlmServiceError::OpenAIError(e))?;

        retry(ExponentialBackoff::default(), || async {
            Ok(self.client.chat().create(request.clone()).await?)
        })
        .await
        .map_err(LlmServiceError::OpenAIError)?;

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

fn role_message_to_request_message(
    role: &LlmRole,
    message: &str,
) -> Result<ChatCompletionRequestMessage, OpenAIError> {
    match role {
        LlmRole::System => ChatCompletionRequestSystemMessageArgs::default()
            .content(message)
            .build()
            .map(|e| ChatCompletionRequestMessage::System(e)),
        LlmRole::User => ChatCompletionRequestUserMessageArgs::default()
            .content(message)
            .build()
            .map(|e| ChatCompletionRequestMessage::User(e)),

        LlmRole::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
            .content(message)
            .build()
            .map(|e| ChatCompletionRequestMessage::Assistant(e)),

        LlmRole::Tool => ChatCompletionRequestToolMessageArgs::default()
            .content(message)
            .build()
            .map(|e| ChatCompletionRequestMessage::Tool(e)),

        LlmRole::Function => ChatCompletionRequestFunctionMessageArgs::default()
            .content(message)
            .build()
            .map(|e| ChatCompletionRequestMessage::Function(e)),
    }
}
