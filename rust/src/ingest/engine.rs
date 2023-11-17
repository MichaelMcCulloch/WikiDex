use super::{
    Ingest,
    IngestError::{self, *},
};
use crate::{embed::Embedder, llm::OpenAiService};
use actix_web::cookie::time::format_description::modifier::Year;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use flate2::{read::GzDecoder, write::GzEncoder};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use parse_mediawiki_dump_reboot::{schema::Namespace, Page};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::{
    rusqlite::{params, DatabaseName},
    SqliteConnectionManager,
};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
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
        connection: &PooledConnection<SqliteConnectionManager>,
        multi_progress: MultiProgress,
    ) -> Result<usize, <Self as Ingest>::E> {
        let access_date = Self::get_date_from_xml_name(input_xml)?;
        let file = BufReader::with_capacity(
            2 * 1024 * 1024,
            File::open(input_xml.as_ref()).map_err(IoError)?,
        );
        let eligible_pages = Self::get_eligible_pages(file, &multi_progress);

        let pages_compressed = Self::compress_articles(eligible_pages, &multi_progress);
        let article_count = pages_compressed.len();
        Self::populate_markup_db(connection, pages_compressed, access_date, &multi_progress)?;

        let naive_utc = chrono::Utc::now().naive_utc();
        connection
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
                let multi_progress = MultiProgress::new();

                let markup_db_path = output_directory.as_ref().join(MARKUP_DB_NAME);

                let connection = Self::get_sqlite_pool(&markup_db_path)
                    .map_err(R2D2Error)?
                    .get()
                    .map_err(R2D2Error)?;
                if !Self::markup_database_is_complete(&connection)? {
                    log::info!("Preparing Markup DB...");
                    self.create_markup_database(input_xml, &connection, multi_progress)?;
                }
                log::info!("Markup DB is ready at {}", markup_db_path.display());

                Ok(1)
            }
        }
    }
}

impl Engine {
    fn compress_articles(
        eligible_pages: Vec<Page>,
        multi_progress: &MultiProgress,
    ) -> Vec<(Vec<u8>, String)> {
        let progress_bar_compress_text =
            multi_progress.add(ProgressBar::new(eligible_pages.len() as u64));

        let pages_compressed = eligible_pages
            .into_par_iter()
            .enumerate()
            .filter_map(|(i, Page { text, title, .. })| {
                if i % 1000 == 0 {
                    progress_bar_compress_text.inc(1000);
                }

                match Self::compress_text(&text) {
                    Ok(compressed) => Some((compressed, title)),
                    Err(_) => None,
                }
            })
            .collect::<Vec<_>>();
        let article_count = pages_compressed.len();
        pages_compressed
    }
    fn populate_markup_db(
        connection: &PooledConnection<SqliteConnectionManager>,
        pages_compressed: Vec<(Vec<u8>, String)>,
        access_date: NaiveDateTime,
        multi_progress: &MultiProgress,
    ) -> Result<(), <Engine as Ingest>::E> {
        let progress_bar = multi_progress.add(ProgressBar::new(pages_compressed.len() as u64));
        Self::pool_execute(&connection, "BEGIN;")?;
        Self::init_markup_sqlite_pool(&connection)?;
        for (i, (text, title)) in pages_compressed.into_iter().enumerate() {
            connection
                .execute(
                    "INSERT INTO wiki_markup (title, text, access_date) VALUES ($1, $2, $3)",
                    (&title, &text, access_date.timestamp_millis()),
                )
                .map_err(RuSqliteError)?;

            if i % 1000 == 0 {
                progress_bar.inc(1000);
            }
        }
        Self::pool_execute(&connection, "COMMIT;")?;
        Ok(())
    }
    fn markup_database_is_complete(
        connection: &PooledConnection<SqliteConnectionManager>,
    ) -> Result<bool, IngestError> {
        let stmt_read_completed_on = connection.prepare(MARKUP_TABLE_IS_COMPLETE_QUERY);
        match stmt_read_completed_on {
            Ok(mut stmt_read_completed_on) => {
                let mapped = stmt_read_completed_on
                    .query_map(params![], |row| {
                        let completed_timestamp: usize = row.get("db_date")?;
                        Ok(completed_timestamp)
                    })
                    .map_err(RuSqliteError)?
                    .filter_map(|f| f.ok())
                    .collect::<Vec<_>>();
                Ok(mapped.len() == 1)
            }
            Err(_) => Ok(false),
        }
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
        r2d2::Pool::builder().max_size(1).build(manager)
    }

    fn init_markup_sqlite_pool(
        connection: &PooledConnection<SqliteConnectionManager>,
    ) -> Result<(), <Self as Ingest>::E> {
        for query in MARKUP_TABLE_CREATE_QUERIES.iter() {
            Self::pool_execute(&connection, query)?;
        }

        connection
            .pragma_update(Some(DatabaseName::Main), "journal_mode", "WAL")
            .map_err(RuSqliteError)?;

        Ok(())
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

    fn get_eligible_pages(file: BufReader<File>, multi_progress: &MultiProgress) -> Vec<Page> {
        let progress_bar = multi_progress.add(ProgressBar::new(10000));

        let parse = parse_mediawiki_dump_reboot::parse(file);
        let eligible_pages = parse
            .filter_map(Result::ok)
            .filter(Self::page_filter)
            .take(10001)
            .enumerate()
            .map(|(i, page)| {
                if i % 1000 == 0 {
                    progress_bar.inc(1000);
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
