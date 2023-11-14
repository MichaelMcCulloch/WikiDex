mod error;
mod protocol;
mod service;
mod vllm;

pub(crate) use error::LlmServiceError;
pub(crate) use protocol::{LlmInput, LlmMessage, LlmRole};
pub(crate) use service::LlmService;
pub(crate) use vllm::OpenAiService;
