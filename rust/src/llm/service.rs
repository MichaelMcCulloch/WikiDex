use super::{protocol::PartialLlmMessage, LlmInput, LlmMessage};
use std::error::Error;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) trait LlmServiceImpl {}

#[async_trait::async_trait]
pub(crate) trait AsyncLlmService {
    type E: Error;
    async fn get_llm_answer(
        &self,
        input: LlmInput,
        max_new_tokens: Option<u16>,
    ) -> Result<LlmMessage, Self::E>;
    async fn stream_llm_answer(
        &self,
        input: LlmInput,
        max_new_tokens: Option<u16>,
        tx: UnboundedSender<PartialLlmMessage>,
    ) -> Result<(), Self::E>;
    async fn wait_for_service(&self) -> Result<(), Self::E>;
}
