use std::error::Error;

use super::LlmInput;

#[async_trait::async_trait]
pub(crate) trait LlmService {
    type E: Error;
    async fn get_llm_answer(
        &self,
        input: LlmInput,
        max_new_tokens: Option<u16>,
    ) -> Result<LlmInput, Self::E>;
}
