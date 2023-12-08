use std::fmt::Display;

use async_openai::{
    error::OpenAIError,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestFunctionMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs, Role,
    },
};
use serde::{Deserialize, Serialize};

use super::{
    AsyncLlmService, AsyncOpenAiService, LlmServiceError, SyncLlmService, SyncOpenAiService,
};
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LlmInput {
    pub(crate) system: String,
    pub(crate) conversation: Vec<LlmMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LlmRole {
    Assistant,
    User,
    System,
    Function,
    Tool,
}

impl Into<Result<Vec<LlmMessage>, <SyncOpenAiService as SyncLlmService>::E>> for LlmInput {
    fn into(self) -> Result<Vec<LlmMessage>, <SyncOpenAiService as SyncLlmService>::E> {
        let Self {
            system,
            conversation,
        } = self;

        let mut messages = vec![LlmMessage {
            role: LlmRole::System,
            content: system,
        }];

        messages.extend(conversation);
        Ok(messages)
    }
}
impl Into<Result<Vec<ChatCompletionRequestMessage>, <AsyncOpenAiService as AsyncLlmService>::E>>
    for LlmInput
{
    fn into(
        self,
    ) -> Result<Vec<ChatCompletionRequestMessage>, <AsyncOpenAiService as AsyncLlmService>::E> {
        let LlmInput {
            system,
            conversation,
        } = self;

        let system_openai_compat =
            role_message_to_request_message(&LlmRole::System, system.as_str())
                .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;

        let mut message_openai_compat = vec![system_openai_compat];

        for message in conversation.iter() {
            let LlmMessage {
                role,
                content: message,
            } = message;
            let msg: ChatCompletionRequestMessage = role_message_to_request_message(&role, message)
                .map_err(|e| LlmServiceError::AsyncOpenAiError(e))?;
            message_openai_compat.push(msg);
        }
        Ok(message_openai_compat)
    }
}

impl Display for LlmRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmRole::Assistant => write!(f, "assistant"),
            LlmRole::User => write!(f, "user"),
            LlmRole::System => write!(f, "system"),
            LlmRole::Function => write!(f, "function"),
            LlmRole::Tool => write!(f, "tool"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LlmMessage {
    pub(crate) role: LlmRole,
    pub(crate) content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PartialLlmMessage {
    pub(crate) role: Option<LlmRole>,
    pub(crate) content: Option<String>,
}

impl From<&Role> for LlmRole {
    fn from(value: &Role) -> Self {
        match value {
            Role::User => LlmRole::User,
            Role::Assistant => LlmRole::Assistant,
            Role::System => LlmRole::System,
            Role::Function => LlmRole::Function,
            Role::Tool => LlmRole::Tool,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialize_to_oaij_format() {
        let input = LlmInput {
            system: String::from("The system message"),
            conversation: vec![
                LlmMessage {
                    role: LlmRole::User,
                    content: String::from("User String"),
                },
                LlmMessage {
                    role: LlmRole::Assistant,
                    content: String::from("User String"),
                },
            ],
        };

        let compat: Result<Vec<LlmMessage>, <AsyncOpenAiService as AsyncLlmService>::E> =
            input.into();

        println!("{}", serde_json::to_string(&compat.unwrap()).unwrap())
    }
}
