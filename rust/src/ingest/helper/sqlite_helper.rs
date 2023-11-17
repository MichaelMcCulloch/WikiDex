use super::{
    super::{Engine, Ingest, IngestError::*},
    wiki::CompressedPage,
};
use chrono::NaiveDateTime;
use indicatif::ProgressBar;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::{
    rusqlite::{params, DatabaseName},
    SqliteConnectionManager,
};
use std::path::Path;

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
    pages_compressed: Vec<CompressedPage>,
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

pub(crate) fn database_is_complete(
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
            "DROP TABLE IF EXISTS wiki_markup;",
            "CREATE TABLE IF NOT EXISTS wiki_markup ( id INTEGER PRIMARY KEY NOT NULL, title BLOB NOT NULL, text BLOB NOT NULL, access_date INTEGER );",
            "CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER, article_count INTEGER);",
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
            "DROP TABLE IF EXISTS completed_on;",
            "DROP TABLE IF EXISTS document;",
            "DROP TABLE IF EXISTS article;",
            "CREATE TABLE IF NOT EXISTS article ( id INTEGER PRIMARY KEY NOT NULL, title BLOB NOT NULL, access_date INTEGER, modification_date INTEGER );",
            "CREATE TABLE IF NOT EXISTS document ( id INTEGER PRIMARY KEY NOT NULL, text BLOB NOT NULL, article INTEGER, FOREIGN KEY(article) REFERENCES article(title)  );",
            "CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER, article_count INTEGER);",
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
