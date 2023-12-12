use std::fmt::Display;

use async_openai::types::Role;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LlmRole {
    Assistant,
    User,
    System,
    Function,
    Tool,
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
