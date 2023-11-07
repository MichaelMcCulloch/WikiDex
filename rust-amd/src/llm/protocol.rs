use std::fmt::Display;

use async_openai::types::Role;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct LlmInput {
    pub(crate) system: String,
    pub(crate) conversation: Vec<LlmMessage>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]

pub(crate) enum LlmRole {
    #[serde(alias = "assistant")]
    Assistant,
    User,
    System,
    Function,
}

impl Display for LlmRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmRole::Assistant => write!(f, "assistant"),
            LlmRole::User => write!(f, "user"),
            LlmRole::System => write!(f, "system"),
            LlmRole::Function => write!(f, "function"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct LlmMessage {
    pub(crate) role: LlmRole,
    pub(crate) message: String,
}

impl Into<Role> for &LlmRole {
    fn into(self) -> Role {
        match *self {
            LlmRole::Assistant => Role::Assistant,
            LlmRole::User => Role::User,
            LlmRole::System => Role::System,
            LlmRole::Function => Role::Function,
        }
    }
}

impl From<&Role> for LlmRole {
    fn from(value: &Role) -> Self {
        match value {
            Role::User => LlmRole::User,
            Role::Assistant => LlmRole::Assistant,
            Role::System => LlmRole::System,
            Role::Function => LlmRole::Function,
        }
    }
}
