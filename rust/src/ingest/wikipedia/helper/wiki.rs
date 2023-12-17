use crate::ingest::wikipedia::IngestError;

use super::{
    super::{
        markup_processor::{self, Process},
        Engine, Ingest,
        IngestError::*,
    },
    gzip_helper::{compress_text, decompress_text},
    text::RecursiveCharacterTextSplitter,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use indicatif::ProgressBar;
use markup_processor::WikiMarkupProcessor;
use parse_mediawiki_dump_reboot::{schema::Namespace, Page};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    fs::File,
    io::BufReader,
    path::Path,
    sync::{mpsc::channel, Arc},
    thread,
    time::Duration,
};

pub(crate) fn get_date_from_xml_name<P: AsRef<Path>>(
    file_name: &P,
) -> Result<NaiveDateTime, IngestError> {
    let date_index_from_split = 1;
    let year_range = 0..4;
    let month_range = 4..6;
    let day_range = 6..8;

    file_name
        .as_ref()
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .and_then(|file_name| Some(file_name.split('-').collect::<Vec<_>>()))
        .and_then(|split| split.get(date_index_from_split).cloned())
        .and_then(|date| if !date.len() == 8 { None } else { Some(date) })
        .and_then(|date| {
            str::parse(&date[year_range])
                .and_then(|y| Ok((y, date)))
                .ok()
        })
        .and_then(|(y, date)| {
            str::parse(&date[month_range])
                .and_then(|m| Ok((y, m, date)))
                .ok()
        })
        .and_then(|(y, m, date)| {
            str::parse(&date[day_range])
                .and_then(|d| Ok((y, m, d)))
                .ok()
        })
        .and_then(|(y, m, d)| {
            NaiveTime::from_num_seconds_from_midnight_opt(0, 0)
                .and_then(|midnight| Some((y, m, d, midnight)))
        })
        .and_then(|(year, month, day, midnight)| {
            NaiveDate::from_ymd_opt(year, month, day).and_then(|d| Some(d.and_time(midnight)))
        })
        .ok_or(XmlDateReadError)
}

pub(crate) fn page_filter(page: &Page) -> bool {
    !page.text.is_empty()
        && page.namespace == Namespace::Main
        && page
            .format
            .as_ref()
            .is_some_and(|format| format == "text/x-wiki")
        && page.model.as_ref().is_some_and(|model| model == "wikitext")
        && !(page.text.starts_with("#REDIRECT") || page.text.starts_with("#redirect"))
}

pub(crate) fn get_eligible_pages(file: BufReader<File>, progress_bar: &ProgressBar) -> Vec<Page> {
    let parse = parse_mediawiki_dump_reboot::parse(file);
    progress_bar.set_message("Getting markup from XML...");
    let eligible_pages = parse
        .filter_map(Result::ok)
        .filter(page_filter)
        .map(|page| {
            progress_bar.inc(1);
            page
        })
        .collect::<Vec<_>>();
    progress_bar.set_message("Getting markup from XML...DONE");
    eligible_pages
}

pub(crate) struct CompressedPage {
    pub(crate) gzipped_text: Vec<u8>,
    pub(crate) article_title: String,
}

pub(crate) struct CompressedPageWithAccessDate {
    pub(crate) gzipped_text: Vec<u8>,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
}

pub(crate) struct DocumentFragments {
    pub(crate) documents: Vec<Vec<u8>>,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}

pub(crate) fn compress_articles(
    eligible_pages: Vec<Page>,
    progress_bar: &ProgressBar,
) -> Vec<CompressedPage> {
    progress_bar.set_message("Compressing Markup...");
    let pages_compressed = eligible_pages
        .into_par_iter()
        .filter_map(|Page { text, title, .. }| {
            progress_bar.inc(1);

            match compress_text(&text) {
                Ok(gzipped_text) => Some(CompressedPage {
                    gzipped_text,
                    article_title: title,
                }),
                Err(_) => None,
            }
        })
        .collect::<Vec<_>>();
    progress_bar.set_message("Compressing Markup...DONE");
    pages_compressed
}

pub(crate) fn decompress_articles_into_documents(
    compressed_pages: Vec<CompressedPageWithAccessDate>,
    progress_bar: &ProgressBar,
    markup_processor: &WikiMarkupProcessor,
    splitter: &RecursiveCharacterTextSplitter,
    minimum_passage_length: usize,
) -> Vec<DocumentFragments> {
    progress_bar.set_message("Decompressing Markup...");

    let markup_processor = Arc::new(markup_processor.clone());

    let documents = compressed_pages
        .into_par_iter()
        .filter_map(
            |CompressedPageWithAccessDate {
                 gzipped_text,
                 article_title,
                 access_date,
             }| {
                let markup_processor = markup_processor.clone();
                let markup = decompress_text(gzipped_text).ok()?;
                let (tx, rx) = channel();
                thread::spawn(move || {
                    let document = markup_processor.process(&markup);

                    let _ = tx.send(document);
                });

                let document = match rx.recv_timeout(Duration::from_secs(60)) {
                    Ok(Ok(document)) => Ok(document),
                    Ok(Err(e)) => Err(MarkupError(e)),
                    Err(_) => {
                        let timeout = Timeout(article_title.clone());
                        log::warn!("{timeout}");
                        Err(timeout)
                    }
                }
                .ok()?;

                progress_bar.inc(1);
                let documents: Vec<Vec<u8>> = splitter
                    .split_text(&document)
                    .into_iter()
                    .filter(|passage| {
                        passage.split(" ").collect::<Vec<_>>().len() > minimum_passage_length
                    })
                    .filter_map(|document| compress_text(&document).map_err(IoError).ok())
                    .collect::<Vec<_>>();
                Some(DocumentFragments {
                    documents,
                    article_title,
                    access_date,
                    modification_date: access_date,
                })
            },
        )
        .collect::<Vec<_>>();

    progress_bar.set_message("Decompressing Markup...DONE");
    documents
}
