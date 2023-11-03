use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};

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

        let mut message_openai_compat = vec![system_openai_compat];

        for message in conversation.iter() {
            let LlmMessage { role, message } = message;
            let role: Role = role.into();
            let msg = ChatCompletionRequestMessageArgs::default()
                .role(role)
                .content(message)
                .build()
                .map_err(|e| LlmServiceError::OpenAIError(e))?;
            message_openai_compat.push(msg);
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
            (_, None) => Err(LlmServiceError::EmptyResponse),
            (Role::System, _) => Err(LlmServiceError::UnexpectedRole(LlmRole::System)),
            (Role::Function, _) => Err(LlmServiceError::UnexpectedRole(LlmRole::Function)),
            (role, Some(message)) => Ok(LlmMessage {
                role: LlmRole::from(&role),
                message,
            }),
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
