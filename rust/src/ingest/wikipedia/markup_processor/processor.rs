use super::{
    super::configurations::WIKIPEDIA_CONFIGURATION, super::helper::wiki::DescribedTable, nodes,
    regexes::Regexes, Process, WikiMarkupProcessingError,
};
use crate::{
    ingest::wikipedia::helper::wiki::UnlabledDocument,
    llm::{LlmInput, LlmService, LlmServiceError, OpenAiService},
};
use actix_rt::ArbiterHandle;
use parse_wiki_text::Configuration;
use std::sync::Arc;

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
    async fn process(&self, markup: &str) -> Result<UnlabledDocument, Self::E> {
        let regexes = Regexes::new();
        let configuration = Configuration::new(WIKIPEDIA_CONFIGURATION);
        let parse = configuration.parse(markup).nodes;
        let process = nodes::process_to_article(&parse, &regexes, &self.llm).await;
        todo!()
    }
}

pub(crate) async fn process_table_to_llm(
    table: &str,
    client: &OpenAiService,
) -> Result<UnlabledDocument, LlmServiceError> {
    let message = LlmInput {
        system: todo!(),
        conversation: todo!(),
    };
    client.get_llm_answer(message).await?;
}
