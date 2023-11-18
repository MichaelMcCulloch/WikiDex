use crate::llm::{AsyncOpenAiService, LlmInput, LlmServiceError, SyncOpenAiService};

use super::{
    super::{configurations::WIKIPEDIA_CONFIGURATION, helper::wiki::UnlabledDocument},
    parse::{process_to_article, Regexes},
    Process, WikiMarkupProcessingError,
};

use parse_wiki_text::Configuration;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct WikiMarkupProcessor {
    llm: Arc<SyncOpenAiService>,
}
impl WikiMarkupProcessor {
    pub(crate) fn new(llm: SyncOpenAiService) -> Self {
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
        let process = process_to_article(&parse, &regexes, &self.llm).await;
        process
    }
}
