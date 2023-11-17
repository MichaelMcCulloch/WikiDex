use actix_web::cookie::time::format_description::modifier::Year;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use flate2::{read::GzDecoder, write::GzEncoder};
use parse_mediawiki_dump_reboot::{schema::Namespace, Page};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::{
    rusqlite::{params, DatabaseName},
    SqliteConnectionManager,
};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use super::{
    Ingest,
    IngestError::{self, *},
};
use crate::{embed::Embedder, llm::OpenAiService};
use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    num::ParseIntError,
    path::{Path, PathBuf},
    time::Instant,
};

const MARKUP_DB_NAME: &str = "wikipedia_markup.sqlite";
const MARKUP_TABLE_CREATE_QUERIES: [&str; 3] = [
    "DROP TABLE IF EXISTS wiki_markup",
    "CREATE TABLE IF NOT EXISTS wiki_markup ( id INTEGER PRIMARY KEY NOT NULL, title BLOB NOT NULL, text BLOB NOT NULL, access_date INTEGER )",
    "CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER, article_count INTEGER)",
];

const MARKUP_TABLE_IS_COMPLETE_QUERY: &str = "SELECT * FROM completed_on;";
const MARKUP_TABLE_MARK_COMPLETE_QUERY: &str =
    "INSERT INTO completed_on (db_date, article_count) VALUES ($1, $2);";

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
        output_directory: &P,
    ) -> Result<usize, <Self as Ingest>::E> {
        let access_date = Self::get_date_from_xml_name(input_xml)?;
        log::info!("{access_date}");
        let file = BufReader::with_capacity(
            2 * 1024 * 1024,
            File::open(input_xml.as_ref()).map_err(IoError)?,
        );

        let eligible_pages = Self::get_eligible_pages(file);
        let pool = Self::get_markup_sqlite_pool(&output_directory.as_ref().join(MARKUP_DB_NAME))?;
        let pool_connection = pool.get().map_err(R2D2Error)?;

        let pages_compressed = eligible_pages
            .into_par_iter()
            .filter_map(|Page { text, title, .. }| {
                let compressed = Self::compress_text(&text).ok();
                Some((compressed, title))
            })
            .collect::<Vec<_>>();

        Self::pool_execute(&pool_connection, "BEGIN;")?;

        let article_count = pages_compressed.len();
        for (i, (text, title)) in pages_compressed.into_iter().enumerate() {
            pool_connection
                .execute(
                    "INSERT INTO wiki_markup (title, text, access_date) VALUES ($1, $2, $3)",
                    (&title, &text, access_date.timestamp_millis()),
                )
                .map_err(RuSqliteError)?;

            if i % 1000 == 0 {
                log::info!("Processed {i} of approximately 7M",);
            }
        }
        Self::pool_execute(&pool_connection, "COMMIT;")?;

        let naive_utc = chrono::Utc::now().naive_utc();
        pool.get()
            .map_err(R2D2Error)?
            .execute(
                MARKUP_TABLE_MARK_COMPLETE_QUERY,
                [naive_utc.timestamp_millis(), article_count as i64],
            )
            .map_err(RuSqliteError)?;
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
                if !Self::markup_database_is_complete(output_directory)? {
                    self.create_markup_database(input_xml, output_directory)?;
                }

                Ok(1)
            }
        }
    }
}

impl Engine {
    fn markup_database_is_complete<P: AsRef<Path>>(
        output_directory: &P,
    ) -> Result<bool, IngestError> {
        let path = output_directory.as_ref().join(MARKUP_DB_NAME);
        if !path.exists() {
            return Ok(false);
        }
        let pool = Self::get_sqlite_pool(&path).map_err(R2D2Error)?;
        let pool_connection = pool.get().map_err(R2D2Error)?;
        let mut stmt_read_completed_on = pool_connection
            .prepare(MARKUP_TABLE_IS_COMPLETE_QUERY)
            .map_err(RuSqliteError)?;
        let mapped = stmt_read_completed_on
            .query_map(params![], |row| {
                let completed_timestamp: usize = row.get("db_date")?;
                Ok(completed_timestamp)
            })
            .map_err(RuSqliteError)?
            .filter_map(|f| f.ok())
            .collect::<Vec<_>>();
        pool_connection.flush_prepared_statement_cache();
        log::info!("{}", mapped.len());
        Ok(mapped.len() == 1)
    }

    // All this to not write XmlDateReadError so many times
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

    fn get_sqlite_pool<P: AsRef<Path>>(
        path: &P,
    ) -> Result<Pool<SqliteConnectionManager>, r2d2::Error> {
        let manager = SqliteConnectionManager::file(path);
        r2d2::Pool::builder().max_size(32).build(manager)
    }

    fn get_markup_sqlite_pool<P: AsRef<Path>>(
        path: &P,
    ) -> Result<Pool<SqliteConnectionManager>, <Self as Ingest>::E> {
        let pool = Self::get_sqlite_pool(path).map_err(R2D2Error)?;

        let pool_connection = pool.get().map_err(R2D2Error)?;
        for query in MARKUP_TABLE_CREATE_QUERIES.iter() {
            Self::pool_execute(&pool_connection, query)?;
        }

        pool.get()
            .map_err(R2D2Error)?
            .pragma_update(Some(DatabaseName::Main), "journal_mode", "WAL")
            .map_err(RuSqliteError)?;

        Ok(pool)
    }

    fn pool_execute(
        pool: &PooledConnection<SqliteConnectionManager>,
        query: &str,
    ) -> Result<(), <Self as Ingest>::E> {
        pool.execute_batch(query).map_err(RuSqliteError)
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

    fn get_eligible_pages(file: BufReader<File>) -> Vec<Page> {
        let parse = parse_mediawiki_dump_reboot::parse(file);
        let eligible_pages = parse
            .filter_map(Result::ok)
            .filter(Self::page_filter)
            .take(10001)
            .enumerate()
            .map(|(i, page)| {
                if i % 10000 == 0 {
                    log::info!("Collected {i} of approximately 7M")
                }
                page
            })
            .collect::<Vec<_>>();
        eligible_pages
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
