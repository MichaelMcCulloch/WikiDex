use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use parse_mediawiki_dump_reboot::{schema::Namespace, Page};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use std::{fs::File, io::BufReader};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::sync::oneshot::channel;
use tokio::time::timeout;

use crate::ingest::pipeline::document::DocumentWithHeading;
use crate::ingest::pipeline::wikipedia::WikiMarkupProcessor;
use crate::ingest::pipeline::{HEADING_END, HEADING_START};
use crate::ingest::{
    pipeline::{
        document::Document,
        error::{PipelineError, WikipediaDumpReaderError},
    },
    service::Process,
};

use super::PipelineStep;

pub(crate) struct WikipediaDumpReader {
    markup_processor: Arc<WikiMarkupProcessor>,
    limit: usize,
}

pub(crate) struct WikipediaHeadingSplitter;

impl WikipediaDumpReader {
    pub(crate) fn new(markup_processor: WikiMarkupProcessor, limit: usize) -> Self {
        Self {
            markup_processor: Arc::new(markup_processor),
            limit,
        }
    }
}

impl PipelineStep for WikipediaHeadingSplitter {
    type IN = Document;

    type OUT = DocumentWithHeading;

    type ARG = ();

    async fn transform(input: Self::IN, _arg: &Self::ARG) -> Vec<Self::OUT> {
        let Document {
            document,
            article_title,
            access_date,
            modification_date,
        } = input;
        document
            .split(HEADING_START)
            .map(|s| {
                let split = s.split(HEADING_END).collect::<Vec<_>>();

                match split.len() {
                    2 => {
                        let heading = format!("{}{}", article_title, split.first().unwrap());
                        let text = split.get(1).unwrap().to_string();
                        (heading, text)
                    }
                    1 => {
                        let text = format!("{}{}", article_title, split.first().unwrap());
                        (String::new(), text)
                    }
                    _ => (String::new(), split.join("")),
                }
            })
            .map(|(heading, document)| DocumentWithHeading {
                document: document.trim().to_string(),
                heading,
                article_title: article_title.clone(),
                access_date,
                modification_date,
            })
            .collect::<Vec<_>>()
    }

    fn args(&self) -> Self::ARG {}
    fn name() -> String {
        String::from("WikipediaHeadingSplitter")
    }
}

impl PipelineStep for WikipediaDumpReader {
    type IN = PathBuf;

    type OUT = Document;

    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
    ) -> Result<UnboundedReceiver<Self::OUT>, PipelineError> {
        let (sender, new_receiver) = unbounded_channel::<Self::OUT>();
        let markup_processor = self.markup_processor.clone();
        let limit = self.limit;

        tokio::spawn(async move {
            while let Some(input) = receiver.recv().await {
                log::info!("{}", input.display());
                let date = get_date_from_xml_name(&input)?;
                let file = File::open(input).map_err(|_| {
                    PipelineError::WikipediaDumpReaderError(
                        WikipediaDumpReaderError::ErrorReadingDump,
                    )
                })?;
                let file = BufReader::with_capacity(2 * 1024 * 1024, file);
                let parse = parse_mediawiki_dump_reboot::parse(file);

                let limit = if limit == 0 { usize::MAX } else { limit };

                let pages = parse.filter_map(Result::ok).filter(page_filter).take(limit);
                for Page { text, title, .. } in pages {
                    let markup_processor = markup_processor.clone();
                    let sender = sender.clone();
                    let (tx, rx) = channel();
                    tokio::spawn(async move {
                        tokio::spawn(async move {
                            let document = markup_processor.process(&text);

                            let _ = tx.send(document);
                        });

                        let received = timeout(Duration::from_secs(60), rx).await.map_err(|e| {
                            log::error!("{} took {}", title.clone(), e);
                            WikipediaDumpReaderError::Timeout(title.clone())
                        })?;

                        if let Ok(received) = received {
                            let document = match received {
                                Ok(document) => Ok(document),
                                Err(e) => Err(WikipediaDumpReaderError::MarkupError(e)),
                            }?;

                            let output = Document {
                                document,
                                article_title: title,
                                access_date: date,
                                modification_date: date,
                            };

                            let _ = sender.send(output);
                        }

                        Ok::<(), WikipediaDumpReaderError>(())
                    });
                }
            }
            Ok::<(), PipelineError>(())
        });
        Ok(new_receiver)
    }

    type ARG = ();

    async fn transform(_input: Self::IN, _arg: &Self::ARG) -> Vec<Self::OUT> {
        todo!()
    }

    fn args(&self) -> Self::ARG {}
    fn name() -> String {
        String::from("WikipediaDumpReader")
    }
}

fn get_date_from_xml_name(file_name: &Path) -> Result<NaiveDateTime, WikipediaDumpReaderError> {
    let date_index_from_split = 1;
    let year_range = 0..4;
    let month_range = 4..6;
    let day_range = 6..8;

    file_name
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(|file_name| file_name.split('-').collect::<Vec<_>>())
        .and_then(|split| split.get(date_index_from_split).cloned())
        .and_then(|date| if !date.len() == 8 { None } else { Some(date) })
        .and_then(|date| str::parse(&date[year_range]).map(|y| (y, date)).ok())
        .and_then(|(y, date)| str::parse(&date[month_range]).map(|m| (y, m, date)).ok())
        .and_then(|(y, m, date)| str::parse(&date[day_range]).map(|d| (y, m, d)).ok())
        .and_then(|(y, m, d)| {
            NaiveTime::from_num_seconds_from_midnight_opt(0, 0).map(|midnight| (y, m, d, midnight))
        })
        .and_then(|(year, month, day, midnight)| {
            NaiveDate::from_ymd_opt(year, month, day).map(|d| d.and_time(midnight))
        })
        .ok_or(WikipediaDumpReaderError::XmlDateReadError)
}
fn page_filter(page: &Page) -> bool {
    !page.text.is_empty()
        && page.namespace == Namespace::Main
        && page
            .format
            .as_ref()
            .is_some_and(|format| format == "text/x-wiki")
        && page.model.as_ref().is_some_and(|model| model == "wikitext")
        && !(page.text.starts_with("#REDIRECT") || page.text.starts_with("#redirect"))
}

fn get_eligible_pages(file: BufReader<File>, ingest_limit: usize) -> Vec<Page> {
    let parse = parse_mediawiki_dump_reboot::parse(file);
    let filtered_pages = parse.filter_map(Result::ok).filter(page_filter);

    if ingest_limit == 0 {
        filtered_pages.collect::<Vec<_>>()
    } else {
        filtered_pages.take(ingest_limit).collect::<Vec<_>>()
    }
}
