use actix_web::rt;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use parse_mediawiki_dump_reboot::{schema::Namespace, Page};
use rayon::iter::ParallelIterator;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{fs::File, io::BufReader};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::channel;
use tokio::time::timeout;

use crate::ingest::wikipedia::WikiMarkupProcessor;
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

impl WikipediaDumpReader {
    pub(crate) fn new(markup_processor: WikiMarkupProcessor, limit: usize) -> Self {
        Self {
            markup_processor: Arc::new(markup_processor),
            limit,
        }
    }
}

impl PipelineStep for WikipediaDumpReader {
    type IN = PathBuf;

    type OUT = Document;

    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
        sender: UnboundedSender<Self::OUT>,
    ) -> Result<(), PipelineError> {
        let markup_processor = self.markup_processor.clone();
        let limit = self.limit;

        rt::spawn(async move {
            while let Some(input) = receiver.recv().await {
                log::info!("{}", input.display());
                let date = get_date_from_xml_name(&input)?;
                let file = File::open(input).map_err(|_| {
                    PipelineError::WikipediaDumpReaderError(
                        WikipediaDumpReaderError::ErrorReadingDump,
                    )
                })?;
                let file = BufReader::with_capacity(2 * 1024 * 1024, file);
                let pages: Vec<Page> = get_eligible_pages(file, limit);
                log::info!("{}", pages.len());
                for Page {
                    format: _,
                    model: _,
                    namespace: _,
                    text,
                    title,
                } in pages
                {
                    let markup_processor = markup_processor.clone();
                    let sender = sender.clone();
                    rt::spawn(async move {
                        let (tx, rx) = channel();
                        rt::spawn(async move {
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
        Ok(())
    }
}
fn get_date_from_xml_name(file_name: &PathBuf) -> Result<NaiveDateTime, WikipediaDumpReaderError> {
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
