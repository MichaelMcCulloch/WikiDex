use super::{
    super::{Engine, Ingest, IngestError::*},
    wiki::{CompressedPage, CompressedPageWithAccessDate},
};
use chrono::NaiveDateTime;
use indicatif::ProgressBar;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::{
    rusqlite::{params, DatabaseName},
    SqliteConnectionManager,
};

use std::path::Path;

pub(crate) const COMPLETION_TABLE_NAME: &str = "completed_on";
pub(crate) const MARKUP_DB_WIKI_MARKUP_TABLE_NAME: &str = "wiki_markup";
pub(crate) const DOCSTORE_DB_DOCUMENT_TABLE_NAME: &str = "document";
pub(crate) const DOCSTORE_DB_ARTICLE_TABLE_NAME: &str = "article";

pub(crate) fn write_completion_timestamp(
    connection: &Pool<SqliteConnectionManager>,
    article_count: usize,
) -> Result<(), <Engine as Ingest>::E> {
    let naive_utc = chrono::Utc::now().naive_utc();
    connection
        .get()
        .unwrap()
        .execute(
            format!(
                "INSERT INTO {COMPLETION_TABLE_NAME} (db_date, article_count) VALUES ($1, $2);"
            )
            .as_str(),
            [naive_utc.timestamp_millis(), article_count as i64],
        )
        .map_err(RuSqliteError)?;
    Ok(())
}

pub(crate) fn populate_markup_db(
    connection: &Pool<SqliteConnectionManager>,
    pages_compressed: Vec<CompressedPage>,
    access_date: NaiveDateTime,
    progress_bar: &ProgressBar,
) -> Result<(), <Engine as Ingest>::E> {
    progress_bar.set_message("Writing Compressed Markup to DB...");
    let connection = connection.get().map_err(R2D2Error)?;
    pool_execute(&connection, "BEGIN;")?;
    init_markup_sqlite_pool(&connection)?;
    for CompressedPage {
        gzipped_text,
        article_title,
    } in pages_compressed.into_iter()
    {
        connection
            .execute(
                format!("INSERT INTO {MARKUP_DB_WIKI_MARKUP_TABLE_NAME} (title, text, access_date) VALUES ($1, $2, $3)").as_str(),
                (&article_title, &gzipped_text, access_date.timestamp_millis()),
            )
            .map_err(RuSqliteError)?;

        progress_bar.inc(1);
    }
    pool_execute(&connection, "COMMIT;")?;
    progress_bar.set_message("Writing Compressed Markup to DB...DONE");
    Ok(())
}

pub(crate) fn database_is_complete(
    pool: &Pool<SqliteConnectionManager>,
) -> Result<bool, <Engine as Ingest>::E> {
    let connection = pool.get().map_err(R2D2Error)?;
    let stmt_read_completed_on =
        connection.prepare(format!("SELECT * FROM {COMPLETION_TABLE_NAME};").as_str());
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
pub(crate) fn obtain_markup(
    connection: &Pool<SqliteConnectionManager>,
    progress_bar: &ProgressBar,
) -> Result<Vec<CompressedPageWithAccessDate>, <Engine as Ingest>::E> {
    progress_bar.set_message("Obtaining Markup...");
    let connection = connection.get().map_err(R2D2Error)?;
    let mut stmt_read_wikimarkup = connection
        .prepare(format!("SELECT * FROM {MARKUP_DB_WIKI_MARKUP_TABLE_NAME};").as_str())
        .map_err(RuSqliteError)?;

    let mapped = stmt_read_wikimarkup
        .query_map(params![], |row| {
            let gzipped_text: Vec<u8> = row.get("text")?;
            let article_title: String = row.get("title")?;
            let timestamp_millis: i64 = row.get("access_date")?;
            progress_bar.inc(1);
            Ok(CompressedPageWithAccessDate {
                gzipped_text,
                article_title,
                access_date: NaiveDateTime::from_timestamp_millis(timestamp_millis).unwrap(),
            })
        })
        .map_err(RuSqliteError)?
        .filter_map(|f| f.ok())
        .collect::<Vec<_>>();

    progress_bar.set_message("Obtaining Markup...DONE");
    Ok(mapped)
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
            format!("DROP TABLE IF EXISTS {COMPLETION_TABLE_NAME};").as_str(),
            format!("DROP TABLE IF EXISTS {MARKUP_DB_WIKI_MARKUP_TABLE_NAME};").as_str(),
            format!("CREATE TABLE IF NOT EXISTS {MARKUP_DB_WIKI_MARKUP_TABLE_NAME} ( id INTEGER PRIMARY KEY NOT NULL, title BLOB NOT NULL, text BLOB NOT NULL, access_date INTEGER );").as_str(),
            format!("CREATE TABLE IF NOT EXISTS {COMPLETION_TABLE_NAME} ( db_date INTEGER, article_count INTEGER);").as_str(),
        ] {
            pool_execute(&connection, query)?;
        }

    connection
        .pragma_update(Some(DatabaseName::Main), "journal_mode", "WAL")
        .map_err(RuSqliteError)?;

    Ok(())
}

pub(crate) fn init_docstore_sqlite_pool(
    connection: &PooledConnection<SqliteConnectionManager>,
) -> Result<(), <Engine as Ingest>::E> {
    for query in [
            format!("DROP TABLE IF EXISTS {COMPLETION_TABLE_NAME};").as_str(),
            format!("DROP TABLE IF EXISTS {DOCSTORE_DB_DOCUMENT_TABLE_NAME};").as_str(),
            format!("DROP TABLE IF EXISTS {DOCSTORE_DB_ARTICLE_TABLE_NAME};").as_str(),
            format!("CREATE TABLE IF NOT EXISTS {DOCSTORE_DB_ARTICLE_TABLE_NAME} ( id INTEGER PRIMARY KEY NOT NULL, title BLOB NOT NULL, access_date INTEGER, modification_date INTEGER );").as_str(),
            format!("CREATE TABLE IF NOT EXISTS {DOCSTORE_DB_DOCUMENT_TABLE_NAME} ( id INTEGER PRIMARY KEY NOT NULL, text BLOB NOT NULL, article INTEGER, FOREIGN KEY(article) REFERENCES {DOCSTORE_DB_ARTICLE_TABLE_NAME}(title)  );").as_str(),
            format!("CREATE TABLE IF NOT EXISTS {COMPLETION_TABLE_NAME} ( db_date INTEGER, article_count INTEGER);").as_str(),
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
