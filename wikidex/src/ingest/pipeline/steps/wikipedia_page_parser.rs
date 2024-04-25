use chrono::NaiveDateTime;

use parse_mediawiki_dump_reboot::Page;

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::oneshot::channel;
use tokio::time::timeout;

use crate::ingest::pipeline::error::{ParseMarkupError, PipelineError};

use crate::ingest::pipeline::wikipedia::WikiMarkupProcessor;
use crate::ingest::{pipeline::document::Document, service::Process};

use super::PipelineStep;

pub(crate) struct WikipediaMarkdownParser {
    markup_processor: Arc<WikiMarkupProcessor>,
}

impl WikipediaMarkdownParser {
    pub(crate) fn new(markup_processor: WikiMarkupProcessor) -> Self {
        Self {
            markup_processor: Arc::new(markup_processor),
        }
    }
}
impl PipelineStep for WikipediaMarkdownParser {
    type IN = (Page, NaiveDateTime);
    type ARG = Arc<WikiMarkupProcessor>;

    type OUT = Document;

    async fn transform(input: Self::IN, arg: &Self::ARG) -> Result<Vec<Self::OUT>, PipelineError> {
        let (Page { text, title, .. }, date) = input;

        let markup_processor = arg.clone();
        let ttext = text.clone();
        let (tx, rx) = channel();
        tokio::spawn(async move {
            let document = markup_processor.process(&ttext);

            let _ = tx.send(document);
        });

        let timeout = timeout(Duration::from_secs(2), rx).await;
        let parse = timeout
            .map_err(|_| ParseMarkupError::Timeout(title.clone()))?
            .map_err(|_| ParseMarkupError::None)?
            .map_err(|_| ParseMarkupError::ParseError(title.clone()))?;
        if parse.is_empty() {
            Err(ParseMarkupError::NoContent(title, text))?
        } else {
            Ok(vec![Document {
                document: parse,
                article_title: title,
                access_date: date,
                modification_date: date,
            }])
        }
    }

    fn args(&self) -> Self::ARG {
        self.markup_processor.clone()
    }
    fn name() -> String {
        String::from("Parser")
    }
}
