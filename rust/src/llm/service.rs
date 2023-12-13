use super::{protocol::PartialLlmMessage, LlmMessage};
use std::error::Error;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) trait LlmServiceImpl {}

pub(crate) struct AsyncLlmServiceArguments<'a> {
    pub(crate) system: &'a str,
    pub(crate) documents: &'a str,
    pub(crate) query: &'a str,
    pub(crate) citation_index_begin: usize,
}

#[async_trait::async_trait]
pub(crate) trait AsyncLlmService {
    type E: Error;
    async fn get_llm_answer(
        &self,
        arguments: AsyncLlmServiceArguments,
    ) -> Result<LlmMessage, Self::E>;
    async fn stream_llm_answer(
        &self,
        arguments: AsyncLlmServiceArguments,
        tx: UnboundedSender<PartialLlmMessage>,
    ) -> Result<(), Self::E>;
    async fn wait_for_service(&self) -> Result<(), Self::E>;
}
