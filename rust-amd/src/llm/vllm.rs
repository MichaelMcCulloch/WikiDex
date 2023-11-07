use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use url::Url;

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
}

impl VllmService {
    pub(crate) fn new<S: AsRef<str>>(host: Url, model_name: S) -> Result<Self, url::ParseError> {
        let openai_config = OpenAIConfig::new().with_api_base(host);
        // let openai_config = OpenAIConfig::new().with_api_base(host.join("v1")?);

        let client = Client::with_config(openai_config);
        let model_name = model_name.as_ref().to_string();
        Ok(Self { client, model_name })
    }
}
