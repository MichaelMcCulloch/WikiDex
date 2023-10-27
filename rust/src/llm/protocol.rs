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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct LlmMessage {
    pub(crate) role: LlmRole,
    pub(crate) message: String,
}
