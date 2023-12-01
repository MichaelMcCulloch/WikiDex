use crate::llm::{LlmServiceError, SyncLlmService, SyncOpenAiService};

use super::{
    super::configurations::WIKIPEDIA_CONFIGURATION,
    parse::{process_to_article, Regexes},
    Process, WikiMarkupProcessingError,
};

use parse_wiki_text::Configuration;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct WikiMarkupProcessor;
impl WikiMarkupProcessor {
    pub(crate) fn new() -> Self {
        Self
    }
}
impl Process for WikiMarkupProcessor {
    type E = WikiMarkupProcessingError;
    fn process(&self, markup: &str) -> Result<String, Self::E> {
        let regexes = Regexes::new();
        let configuration = Configuration::new(WIKIPEDIA_CONFIGURATION);
        let parse = configuration.parse(markup).nodes;
        let process = process_to_article(&parse, &regexes);
        process
    }
}
