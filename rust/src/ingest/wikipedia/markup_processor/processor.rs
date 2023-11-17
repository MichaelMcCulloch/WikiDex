use std::sync::Arc;

use actix_rt::ArbiterHandle;

use crate::llm::{LlmInput, LlmService, OpenAiService};

use super::{super::helper::wiki::DescribedTable, Process, WikiMarkupProcessingError};

#[derive(Clone)]
pub(crate) struct WikiMarkupProcessor {
    llm: Arc<OpenAiService>,
}
impl WikiMarkupProcessor {
    pub(crate) fn new(llm: OpenAiService) -> Self {
        Self { llm: Arc::new(llm) }
    }
}
#[async_trait::async_trait]
impl Process for WikiMarkupProcessor {
    type E = WikiMarkupProcessingError;
    async fn process(&self, markup: &str) -> Result<(String, Vec<DescribedTable>), Self::E> {
        todo!()
    }
}

pub(crate) async fn process_table_to_llm(table: &str, client: &OpenAiService) -> String {
    let message = LlmInput {
        system: todo!(),
        conversation: todo!(),
    };
    client.get_llm_answer(message).await;
}
