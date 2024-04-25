use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use indicatif::ProgressBar;
use parse_mediawiki_dump_reboot::{schema::Namespace, Page};
use std::path::{Path, PathBuf};

use std::sync::Arc;
use std::{fs::File, io::BufReader};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use crate::ingest::pipeline::error::{LinkError, PipelineError, WikipediaDumpReaderError};

use super::PipelineStep;

pub(crate) struct WikipediaDumpReader {
    limit: usize,
}

impl WikipediaDumpReader {
    pub(crate) fn new(limit: usize) -> Self {
        Self { limit }
    }
}
impl PipelineStep for WikipediaDumpReader {
    type IN = PathBuf;
    type ARG = ();
    type OUT = (Page, NaiveDateTime);

    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
        progress: Arc<ProgressBar>,
        next_progress: Vec<Arc<ProgressBar>>,
    ) -> Result<Vec<UnboundedReceiver<Self::OUT>>, PipelineError> {
        let (sender, new_receiver) = unbounded_channel::<Self::OUT>();
        let next_progress = next_progress
            .first()
            .ok_or(LinkError::NoCurrentProgressBar)?
            .clone();

        let limit = self.limit;
        progress.set_message(Self::name().to_string());
        progress.set_length(6968500);
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

                for page in pages {
                    next_progress.inc_length(1);
                    let sender = sender.clone();
                    tokio::spawn(async move {
                        let _ = sender.send((page, date));
                    });
                    progress.inc(1);
                }
            }
            Ok::<(), PipelineError>(())
        });
        Ok(vec![new_receiver])
    }

    async fn transform(
        _input: Self::IN,
        _arg: &Self::ARG,
    ) -> Result<Vec<Self::OUT>, PipelineError> {
        unimplemented!()
    }

    fn args(&self) -> Self::ARG {}
    fn name() -> String {
        String::from("Wikipedia Dump Reader")
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
        && !(page.text.starts_with("#REDIRECT")
            || page.text.starts_with("#redirect")
            || page.text.is_empty())
}
