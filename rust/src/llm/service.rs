use std::error::Error;

use url::Url;

use super::{AsyncOpenAiService, LlmInput, LlmMessage, SyncOpenAiService};

pub(crate) trait LlmServiceImpl {}

#[async_trait::async_trait]
pub(crate) trait AsyncLlmService {
    type E: Error;
    async fn get_llm_answer(
        &self,
        input: LlmInput,
        max_new_tokens: Option<u16>,
    ) -> Result<LlmMessage, Self::E>;
    async fn wait_for_service(&self) -> Result<(), Self::E>;
}

pub(crate) trait SyncLlmService {
    type E: Error;
    fn get_llm_answer(
        &self,
        input: LlmInput,
        max_new_tokens: Option<u16>,
    ) -> Result<LlmMessage, Self::E>;

    fn wait_for_service(&self) -> Result<(), Self::E>;
}
