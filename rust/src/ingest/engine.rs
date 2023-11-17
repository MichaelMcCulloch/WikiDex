use super::{
    helper::{
        get_sqlite_pool, markup_database_is_complete, populate_markup_db,
        write_completion_timestamp,
    },
    Ingest,
    IngestError::{self, *},
};
use crate::{embed::Embedder, llm::OpenAiService};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use flate2::{read::GzDecoder, write::GzEncoder};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use parse_mediawiki_dump_reboot::{schema::Namespace, Page};
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    path::Path,
};

const MARKUP_DB_NAME: &str = "wikipedia_markup.sqlite";
const DOCSTORE_DB_NAME: &str = "wikipedia_docstore.sqlite";
const VECTOR_DB_NAME: &str = "wikipedia_index.faiss";

pub(crate) struct Engine {
    embed: Embedder,
    llm: OpenAiService,
    thread_count: usize,
}

impl Engine {
    pub(crate) fn new(embed: Embedder, llm: OpenAiService) -> Self {
        Self {
            embed,
            llm,
            thread_count: 32,
        }
    }

    fn create_markup_database<P: AsRef<Path>>(
        &self,
        input_xml: &P,
        connection: &PooledConnection<SqliteConnectionManager>,
    ) -> Result<usize, <Self as Ingest>::E> {
        let multi_progress = MultiProgress::new();
        let access_date = Self::get_date_from_xml_name(input_xml)?;
        let file = BufReader::with_capacity(
            2 * 1024 * 1024,
            File::open(input_xml.as_ref()).map_err(IoError)?,
        );

        let eligible_pages_bar = new_progress_bar(&multi_progress, 7000000);
        let eligible_pages = Self::get_eligible_pages(file, &eligible_pages_bar);

        let pages_compressed_bar = new_progress_bar(&multi_progress, eligible_pages.len() as u64);
        let pages_compressed = Self::compress_articles(eligible_pages, &pages_compressed_bar);

        let article_count = pages_compressed.len();
        let markup_written_bar = new_progress_bar(&multi_progress, article_count as u64);
        populate_markup_db(
            connection,
            pages_compressed,
            access_date,
            &markup_written_bar,
        )?;

        write_completion_timestamp(connection, article_count)?;
        Ok(article_count)
    }
}

impl Ingest for Engine {
    type E = IngestError;

    fn ingest_wikipedia<P: AsRef<Path>>(
        self,
        input_xml: &P,
        output_directory: &P,
    ) -> Result<usize, Self::E> {
        match (
            input_xml.as_ref().exists(),
            output_directory.as_ref().exists(),
        ) {
            (true, false) => Err(OutputDirectoryNotFound(
                output_directory.as_ref().to_path_buf(),
            )),
            (false, _) => Err(XmlNotFound(input_xml.as_ref().to_path_buf())),
            (true, true) => {
                let markup_db_path = output_directory.as_ref().join(MARKUP_DB_NAME);

                let connection = get_sqlite_pool(&markup_db_path)
                    .and_then(|pool| pool.get())
                    .map_err(R2D2Error)?;

                if !markup_database_is_complete(&connection)? {
                    log::info!("Preparing Markup DB...");
                    self.create_markup_database(input_xml, &connection)?;
                }
                log::info!("Markup DB is ready at {}", markup_db_path.display());

                Ok(1)
            }
        }
    }
}

impl Engine {
    fn get_date_from_xml_name<P: AsRef<Path>>(
        file_name: &P,
    ) -> Result<NaiveDateTime, <Self as Ingest>::E> {
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

    fn get_eligible_pages(file: BufReader<File>, progress_bar: &ProgressBar) -> Vec<Page> {
        let parse = parse_mediawiki_dump_reboot::parse(file);
        progress_bar.set_message("Getting markup from XML...");
        let eligible_pages = parse
            .filter_map(Result::ok)
            .filter(Self::page_filter)
            .take(10001)
            .map(|page| {
                progress_bar.inc(1);
                page
            })
            .collect::<Vec<_>>();
        progress_bar.set_message("Getting markup from XML...DONE");
        eligible_pages
    }

    fn compress_articles(
        eligible_pages: Vec<Page>,
        progress_bar: &ProgressBar,
    ) -> Vec<(Vec<u8>, String)> {
        progress_bar.set_message("Compressing Markup...");
        let pages_compressed = eligible_pages
            .into_par_iter()
            .filter_map(|Page { text, title, .. }| {
                progress_bar.inc(1);

                match Self::compress_text(&text) {
                    Ok(compressed) => Some((compressed, title)),
                    Err(_) => None,
                }
            })
            .collect::<Vec<_>>();
        progress_bar.set_message("Compressing Markup...DONE");
        pages_compressed
    }

    fn compress_text(text: &str) -> Result<Vec<u8>, io::Error> {
        let mut text_compress = vec![];
        {
            let mut encoder = GzEncoder::new(&mut text_compress, flate2::Compression::new(9));
            write!(&mut encoder, "{text}")?;
            encoder.flush()?;
        }
        Ok(text_compress)
    }

    fn decompress_text(text_compressed: &Vec<u8>) -> Result<String, io::Error> {
        let mut text = String::new();
        {
            let mut decoder = GzDecoder::new(&text_compressed[..]);
            decoder.read_to_string(&mut text)?;
        }
        Ok(text)
    }
}

fn new_progress_bar(multibar: &MultiProgress, limit: u64) -> ProgressBar {
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap();

    let pb = multibar.add(ProgressBar::new(limit));
    pb.set_style(sty);
    pb
}
