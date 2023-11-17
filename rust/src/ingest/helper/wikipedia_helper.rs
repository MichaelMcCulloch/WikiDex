use super::{
    super::{Engine, Ingest, IngestError::*},
    gzip_helper::compress_text,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use indicatif::ProgressBar;
use parse_mediawiki_dump_reboot::{schema::Namespace, Page};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{fs::File, io::BufReader, path::Path};

pub(crate) fn get_date_from_xml_name<P: AsRef<Path>>(
    file_name: &P,
) -> Result<NaiveDateTime, <Engine as Ingest>::E> {
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

pub(crate) type CompressedPage = (Vec<u8>, String);

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
                Ok(compressed) => Some((compressed, title)),
                Err(_) => None,
            }
        })
        .collect::<Vec<_>>();
    progress_bar.set_message("Compressing Markup...DONE");
    pages_compressed
}
