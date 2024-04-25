use chrono::NaiveDateTime;

use parse_mediawiki_dump_reboot::Page;

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::oneshot::channel;
use tokio::time::timeout;

use crate::ingest::pipeline::error::{PipelineError, WikipediaMarkupParseError};

use crate::ingest::pipeline::wikipedia::WikiMarkupProcessor;
use crate::ingest::{pipeline::document::Document, service::Process};

use super::PipelineStep;

pub(crate) struct WikipediaMarkdownParser {
    markup_processor: Arc<WikiMarkupProcessor>,
    article_counter: Arc<AtomicI64>,
}

impl WikipediaMarkdownParser {
    pub(crate) fn new(markup_processor: WikiMarkupProcessor) -> Self {
        Self {
            markup_processor: Arc::new(markup_processor),
            article_counter: Arc::new(AtomicI64::new(0)),
        }
    }
}
impl PipelineStep for WikipediaMarkdownParser {
    type IN = (Page, NaiveDateTime);
    type ARG = (Arc<WikiMarkupProcessor>, Arc<AtomicI64>);

    type OUT = Document;

    async fn transform(input: Self::IN, arg: &Self::ARG) -> Result<Vec<Self::OUT>, PipelineError> {
        let (Page { text, title, .. }, date) = input;

        let markup_processor = arg.0.clone();
        let ttext = text.clone();
        let (tx, rx) = channel();
        tokio::spawn(async move {
            let document = markup_processor.process(&ttext);

            let _ = tx.send(document);
        });

        let timeout = timeout(Duration::from_secs(2), rx).await;
        let parse = timeout
            .map_err(|_| WikipediaMarkupParseError::Timeout(title.clone()))?
            .map_err(|_| WikipediaMarkupParseError::None)?
            .map_err(|_| WikipediaMarkupParseError::ParseError(title.clone()))?;
        if parse.is_empty() {
            Err(WikipediaMarkupParseError::NoContent(title, text))?
        } else {
            Ok(vec![Document {
                document: parse,
                article_title: title,
                access_date: date,
                modification_date: date,
                article_id: arg.1.fetch_add(1, Ordering::Relaxed),
            }])
        }
    }

    fn args(&self) -> Self::ARG {
        (self.markup_processor.clone(), self.article_counter.clone())
    }
    fn name() -> String {
        String::from("Parser")
    }
}
