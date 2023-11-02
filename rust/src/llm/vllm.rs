use async_openai::{
    config::{Config, OpenAIConfig},
    types::{
        ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs,
        CreateCompletionRequestArgs, Role,
    },
    Client,
};
use url::Url;

use crate::config::{ConfigUrl, LlmConfig};

use super::{
    protocol::{LlmInput, LlmMessage, LlmRole},
    LlmService, LlmServiceError,
};

pub(crate) struct VllmService {
    client: Client<OpenAIConfig>,
    model_name: String,
}

#[async_trait::async_trait]
impl LlmService for VllmService {
    type E = LlmServiceError;
    async fn get_llm_answer(&self, input: LlmInput) -> Result<LlmInput, Self::E> {
        let LlmInput {
            system,
            mut conversation,
        } = input;

        let system_openai_compat = ChatCompletionRequestMessageArgs::default()
            .role(Role::System)
            .content(system.clone())
            .build()
            .map_err(|e| LlmServiceError::OpenAIError(e))?;

        let mut message_openai_compat = vec![];
        message_openai_compat.push(system_openai_compat);
        for message in conversation.iter() {
            let m = match message {
                LlmMessage {
                    role: LlmRole::Assistant,
                    message,
                } => ChatCompletionRequestMessageArgs::default()
                    .role(Role::Assistant)
                    .content(message)
                    .build()
                    .map_err(|e| LlmServiceError::OpenAIError(e))?,
                LlmMessage {
                    role: LlmRole::User,
                    message,
                } => ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content(message)
                    .build()
                    .map_err(|e| LlmServiceError::OpenAIError(e))?,
            };
            message_openai_compat.push(m);
        }

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(2048u16)
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
        let response = match (response.message.role, response.message.content) {
            (Role::User, Some(message)) => Ok(LlmMessage {
                role: LlmRole::User,
                message,
            }),
            (Role::Assistant, Some(message)) => Ok(LlmMessage {
                role: LlmRole::Assistant,
                message,
            }),
            _ => Err(LlmServiceError::UnexpectedResponse),
        }?;

        conversation.push(response);
        Ok(LlmInput {
            system,
            conversation,
        })
    }
}

impl VllmService {
    pub(crate) fn new(config: LlmConfig) -> Result<Self, url::ParseError> {
        let openai_config = OpenAIConfig::new().with_api_base(config.url().join("v1")?);

        let client = Client::with_config(openai_config);

        Ok(Self {
            client,
            model_name: config.model,
        })
    }
}
