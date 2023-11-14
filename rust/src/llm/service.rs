use std::error::Error;

use super::LlmInput;

#[async_trait::async_trait]
pub(crate) trait LlmService {
    type E: Error;
    async fn get_llm_answer(&self, input: LlmInput) -> Result<LlmInput, Self::E>;
}
