use super::{Engine, Ingest, IngestError::*};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use flate2::{read::GzDecoder, write::GzEncoder};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use parse_mediawiki_dump_reboot::{schema::Namespace, Page};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::{
    rusqlite::{params, DatabaseName},
    SqliteConnectionManager,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    path::Path,
};

pub(crate) fn write_completion_timestamp(
    connection: &PooledConnection<SqliteConnectionManager>,
    article_count: usize,
) -> Result<(), <Engine as Ingest>::E> {
    let naive_utc = chrono::Utc::now().naive_utc();
    connection
        .execute(
            "INSERT INTO completed_on (db_date, article_count) VALUES ($1, $2);",
            [naive_utc.timestamp_millis(), article_count as i64],
        )
        .map_err(RuSqliteError)?;
    Ok(())
}

pub(crate) fn populate_markup_db(
    connection: &PooledConnection<SqliteConnectionManager>,
    pages_compressed: Vec<(Vec<u8>, String)>,
    access_date: NaiveDateTime,
    progress_bar: &ProgressBar,
) -> Result<(), <Engine as Ingest>::E> {
    progress_bar.set_message("Writing Compressed Markup to DB...");
    pool_execute(&connection, "BEGIN;")?;
    init_markup_sqlite_pool(&connection)?;
    for (text, title) in pages_compressed.into_iter() {
        connection
            .execute(
                "INSERT INTO wiki_markup (title, text, access_date) VALUES ($1, $2, $3)",
                (&title, &text, access_date.timestamp_millis()),
            )
            .map_err(RuSqliteError)?;

        progress_bar.inc(1);
    }
    pool_execute(&connection, "COMMIT;")?;
    progress_bar.set_message("Writing Compressed Markup to DB...DONE");
    Ok(())
}

pub(crate) fn markup_database_is_complete(
    connection: &PooledConnection<SqliteConnectionManager>,
) -> Result<bool, <Engine as Ingest>::E> {
    let stmt_read_completed_on = connection.prepare("SELECT * FROM completed_on;");
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

pub(crate) fn get_sqlite_pool<P: AsRef<Path>>(
    path: &P,
) -> Result<Pool<SqliteConnectionManager>, r2d2::Error> {
    let manager = SqliteConnectionManager::file(path);
    r2d2::Pool::builder().max_size(1).build(manager)
}

pub(crate) fn init_markup_sqlite_pool(
    connection: &PooledConnection<SqliteConnectionManager>,
) -> Result<(), <Engine as Ingest>::E> {
    for query in [
            "DROP TABLE IF EXISTS completed_on;",
            "DROP TABLE IF EXISTS wiki_markup",
            "CREATE TABLE IF NOT EXISTS wiki_markup ( id INTEGER PRIMARY KEY NOT NULL, title BLOB NOT NULL, text BLOB NOT NULL, access_date INTEGER )",
            "CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER, article_count INTEGER)",
        ] {
            pool_execute(&connection, query)?;
        }

    connection
        .pragma_update(Some(DatabaseName::Main), "journal_mode", "WAL")
        .map_err(RuSqliteError)?;

    Ok(())
}

pub(crate) fn pool_execute(
    pool: &PooledConnection<SqliteConnectionManager>,
    query: &str,
) -> Result<(), <Engine as Ingest>::E> {
    pool.execute_batch(query).map_err(RuSqliteError)
}

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

pub(crate) fn compress_articles(
    eligible_pages: Vec<Page>,
    progress_bar: &ProgressBar,
) -> Vec<(Vec<u8>, String)> {
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

pub(crate) fn compress_text(text: &str) -> Result<Vec<u8>, io::Error> {
    let mut text_compress = vec![];
    {
        let mut encoder = GzEncoder::new(&mut text_compress, flate2::Compression::new(9));
        write!(&mut encoder, "{text}")?;
        encoder.flush()?;
    }
    Ok(text_compress)
}

pub(crate) fn decompress_text(text_compressed: &Vec<u8>) -> Result<String, io::Error> {
    let mut text = String::new();
    {
        let mut decoder = GzDecoder::new(&text_compressed[..]);
        decoder.read_to_string(&mut text)?;
    }
    Ok(text)
}

pub(crate) fn new_progress_bar(multibar: &MultiProgress, limit: u64) -> ProgressBar {
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap();

    let pb = multibar.add(ProgressBar::new(limit));
    pb.set_style(sty);
    pb
}
