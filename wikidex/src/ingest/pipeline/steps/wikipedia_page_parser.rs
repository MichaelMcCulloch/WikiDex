use chrono::NaiveDateTime;

use parse_mediawiki_dump_reboot::Page;

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::oneshot::channel;
use tokio::time::timeout;

use crate::ingest::pipeline::error::{ParseError, PipelineError};
use crate::ingest::pipeline::wikipedia::WikiMarkupProcessor;

use crate::ingest::{
    pipeline::{document::Document},
    service::Process,
};

use super::PipelineStep;

pub(crate) struct WikipediaPageParser {
    markup_processor: Arc<WikiMarkupProcessor>,
}

impl WikipediaPageParser {
    pub(crate) fn new(markup_processor: WikiMarkupProcessor) -> Self {
        Self {
            markup_processor: Arc::new(markup_processor),
        }
    }
}
impl PipelineStep for WikipediaPageParser {
    type IN = (Page, NaiveDateTime);
    type ARG = Arc<WikiMarkupProcessor>;

    type OUT = Document;

    async fn transform(input: Self::IN, arg: &Self::ARG) -> Result<Vec<Self::OUT>, PipelineError> {
        let (Page { text, title, .. }, date) = input;

        let markup_processor = arg.clone();
        let (tx, rx) = channel();
        tokio::spawn(async move {
            let document = markup_processor.process(&text);

            let _ = tx.send(document);
        });

        let parse = timeout(Duration::from_secs(60), rx)
            .await
            .map_err(|_| ParseError::Timeout)?
            .map_err(ParseError::Tokio)?
            .map_err(|_| ParseError::ParseError(title.clone()))?;

        let output = Document {
            document: parse,
            article_title: title,
            access_date: date,
            modification_date: date,
        };

        Ok(vec![output])
    }

    fn args(&self) -> Self::ARG {
        self.markup_processor.clone()
    }
    fn name() -> String {
        String::from("WikipediaPageParser")
    }
}
