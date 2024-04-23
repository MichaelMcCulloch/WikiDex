use chrono::{NaiveDateTime};

use parse_mediawiki_dump_reboot::{Page};

use std::sync::Arc;
use std::time::Duration;


use tokio::sync::oneshot::channel;
use tokio::time::timeout;

use crate::ingest::pipeline::wikipedia::WikiMarkupProcessor;

use crate::ingest::{
    pipeline::{
        document::Document,
        error::{WikipediaDumpReaderError},
    },
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

    async fn transform(input: Self::IN, arg: &Self::ARG) -> Vec<Self::OUT> {
        let (Page { text, title, .. }, date) = input;

        let markup_processor = arg.clone();
        let (tx, rx) = channel();
        tokio::spawn(async move {
            let document = markup_processor.process(&text);

            let _ = tx.send(document);
        });

        let received = timeout(Duration::from_secs(60), rx)
            .await
            .map_err(|e| {
                log::error!("{} took {}", title.clone(), e);
                WikipediaDumpReaderError::Timeout(title.clone())
            })
            .unwrap();

        if let Ok(received) = received {
            let document = match received {
                Ok(document) => Ok(document),
                Err(e) => Err(WikipediaDumpReaderError::MarkupError(e)),
            }
            .unwrap();

            let output = Document {
                document,
                article_title: title,
                access_date: date,
                modification_date: date,
            };

            vec![output]
        } else {
            vec![]
        }
    }

    fn args(&self) -> Self::ARG {
        self.markup_processor.clone()
    }
    fn name() -> String {
        String::from("WikipediaPageParser")
    }
}
