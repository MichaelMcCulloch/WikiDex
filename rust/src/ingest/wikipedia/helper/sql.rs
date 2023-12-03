use crate::{
    embed::{sync::Embedder, EmbedServiceSync},
    ingest::wikipedia::IngestError,
};

use super::{
    super::{Engine, Ingest, IngestError::*},
    gzip_helper::{compress_text, decompress_text},
    wiki::{CompressedPage, CompressedPageWithAccessDate, Document, DocumentFragments},
};
use chrono::NaiveDateTime;
use indicatif::ProgressBar;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::{
    rusqlite::{params, DatabaseName},
    SqliteConnectionManager,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use std::{
    io::Write,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender},
    },
};

pub(crate) const COMPLETION_TABLE_NAME: &str = "completed_on";
pub(crate) const MARKUP_DB_WIKI_MARKUP_TABLE_NAME: &str = "wiki_markup";
pub(crate) const DOCSTORE_DB_DOCUMENT_TABLE_NAME: &str = "document";
pub(crate) const DOCSTORE_DB_ARTICLE_TABLE_NAME: &str = "article";
pub(crate) const EMBEDDINGS_DB_EMBEDDINGS_TABLE_NAME: &str = "embeddings";

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
) -> Result<usize, <Engine as Ingest>::E> {
    progress_bar.set_message("Writing Compressed Markup to DB...");
    let connection = connection.get().map_err(R2D2Error)?;
    pool_execute(&connection, "BEGIN;")?;
    init_markup_sqlite_pool(&connection)?;

    let article_count = AtomicUsize::new(0);

    for CompressedPage {
        gzipped_text,
        article_title,
    } in pages_compressed.into_iter()
    {
        let article_id: usize = article_count.fetch_add(1, Ordering::Relaxed);
        connection
            .execute(
                format!("INSERT INTO {MARKUP_DB_WIKI_MARKUP_TABLE_NAME} (id, title, text, access_date) VALUES ($1, $2, $3, $4)").as_str(),
                (&article_id, &article_title, &gzipped_text, access_date.timestamp_millis()),
            )
            .map_err(RuSqliteError)?;

        progress_bar.inc(1);
    }
    pool_execute(&connection, "COMMIT;")?;
    progress_bar.set_message("Writing Compressed Markup to DB...DONE");
    Ok(article_count.fetch_add(0, Ordering::SeqCst))
}

pub(crate) fn populate_docstore_db(
    connection: &Pool<SqliteConnectionManager>,
    pages_compressed: Vec<DocumentFragments>,
    progress_bar: &ProgressBar,
) -> Result<usize, <Engine as Ingest>::E> {
    progress_bar.set_message("Writing Docstore to DB...");
    let connection = connection.get().map_err(R2D2Error)?;
    pool_execute(&connection, "BEGIN;")?;
    init_docstore_sqlite_pool(&connection)?;

    let article_count = AtomicUsize::new(0);
    let document_count = AtomicUsize::new(0);

    for DocumentFragments {
        documents,
        article_title,
        access_date,
        modification_date,
    } in pages_compressed.into_iter()
    {
        let article_id = article_count.fetch_add(1, Ordering::Relaxed);
        connection
            .execute(
                format!("INSERT INTO {DOCSTORE_DB_ARTICLE_TABLE_NAME} (id , title, access_date, modification_date) VALUES ($1, $2, $3, $4)").as_str(),
                (&article_id, &article_title, &access_date.timestamp_millis(), modification_date.timestamp_millis()),
            )
        .map_err(RuSqliteError)?;

        for document in documents {
            let document_id = document_count.fetch_add(1, Ordering::Relaxed);

            connection
            .execute(
                format!("INSERT INTO {DOCSTORE_DB_DOCUMENT_TABLE_NAME} (id, text, article) VALUES ($1, $2, $3)").as_str(),
                (&document_id, &document, &article_id),
            )
            .map_err(RuSqliteError)?;
        }

        progress_bar.inc(1);
    }
    pool_execute(&connection, "COMMIT;")?;
    progress_bar.set_message("Writing Docstore to DB...DONE");
    Ok(document_count.fetch_add(0, Ordering::SeqCst))
}
pub(crate) fn write_vectorstore(
    rx: Receiver<(Vec<usize>, Vec<Vec<f32>>)>,
    tmp_vector_pool_clone: &Pool<SqliteConnectionManager>,
    create_vectors_bar: &ProgressBar,
) -> Result<(), IngestError> {
    let tmp_vector_connection = &tmp_vector_pool_clone.get().map_err(R2D2Error)?;
    init_temp_embedding_sqlite_pool(&tmp_vector_connection)?;

    while let Ok((indices, embeddings)) = rx.recv() {
        tmp_vector_connection
            .execute_batch("BEGIN;")
            .map_err(RuSqliteError)?;
        let count = indices.len();
        for (index, embedding) in indices.into_iter().zip(embeddings) {
            let mut v8: Vec<u8> = vec![];

            for e in embedding {
                v8.write_all(&e.to_le_bytes()).map_err(IoError)?;
            }

            tmp_vector_connection
                            .execute(
                                &format!("INSERT INTO {EMBEDDINGS_DB_EMBEDDINGS_TABLE_NAME} (id, gte_small) VALUES ($1, $2)"),
                                params![index, v8],
                            )
                            .map_err(RuSqliteError)?;
        }
        create_vectors_bar.inc(count as u64);

        tmp_vector_connection
            .execute_batch("COMMIT;")
            .map_err(RuSqliteError)?;
    }
    Ok(())
}
pub(crate) fn populate_vectorstore_db(
    embedder: &Embedder,
    docstore_pool: &Pool<SqliteConnectionManager>,
    document_count: usize,
    tx: Sender<(Vec<usize>, Vec<Vec<f32>>)>,
    batch_size: usize,
) -> Result<(), <Engine as Ingest>::E> {
    let docstore_connection = &docstore_pool.get().map_err(R2D2Error)?;
    Ok(
        for indices in (0..document_count).collect::<Vec<_>>().chunks(batch_size) {
            let mut stmt_read_document = docstore_connection
            .prepare(&format!(
                "SELECT id, text FROM {DOCSTORE_DB_DOCUMENT_TABLE_NAME} WHERE id >= $1 AND id <= $2 ORDER BY id ASC;"
            ))
            .map_err(RuSqliteError)?;

            let start = indices.first().unwrap();
            let end = indices.last().unwrap();

            let mapped = stmt_read_document
                .query_map(params![start, end], |row| {
                    let id: usize = row.get(0)?;
                    let doc: Vec<u8> = row.get(1)?;
                    Ok((id, doc))
                })
                .map_err(RuSqliteError)?
                .filter_map(|f| f.ok())
                .collect::<Vec<_>>();

            let rows = mapped
                .into_par_iter()
                .filter_map(|(id, doc)| Some((id, decompress_text(doc).ok()?)))
                .collect::<Vec<_>>();

            let (ids, batch): (Vec<usize>, Vec<String>) = rows.into_iter().unzip();
            let batch = batch.iter().map(|s| s.as_str()).collect::<Vec<_>>();

            let batch_result = embedder.embed(&batch).map_err(EmbeddingServiceError)?;
            let _ = tx.send((ids, batch_result));
        },
    )
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

pub(crate) fn count_elements(
    pool: &Pool<SqliteConnectionManager>,
) -> Result<Option<usize>, <Engine as Ingest>::E> {
    let connection = &pool.get().map_err(R2D2Error)?;
    let mut count_stmt = connection
        .prepare(&format!(
            "SELECT article_count FROM {COMPLETION_TABLE_NAME} LIMIT 1;"
        ))
        .map_err(RuSqliteError)?;
    let mut rows = count_stmt.query(params![]).map_err(RuSqliteError)?;
    let next_row = rows
        .next()
        .map(|next_row| next_row.map(|row| row.get(0)))
        .map_err(RuSqliteError)?;
    match next_row {
        Some(Ok(count)) => Ok(count),
        Some(e) => e.map_err(RuSqliteError),
        None => Ok(None),
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
            format!("CREATE TABLE IF NOT EXISTS {DOCSTORE_DB_ARTICLE_TABLE_NAME} ( id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, access_date INTEGER, modification_date INTEGER );").as_str(),
            format!("CREATE TABLE IF NOT EXISTS {DOCSTORE_DB_DOCUMENT_TABLE_NAME} ( id INTEGER PRIMARY KEY NOT NULL, text BLOB NOT NULL, article INTEGER, FOREIGN KEY(article) REFERENCES {DOCSTORE_DB_ARTICLE_TABLE_NAME}(id)  );").as_str(),
            format!("CREATE TABLE IF NOT EXISTS {COMPLETION_TABLE_NAME} ( db_date INTEGER, article_count INTEGER);").as_str(),
        ] {
            pool_execute(&connection, query)?;
        }

    connection
        .pragma_update(Some(DatabaseName::Main), "journal_mode", "WAL")
        .map_err(RuSqliteError)?;

    Ok(())
}
pub(crate) fn init_temp_embedding_sqlite_pool(
    connection: &PooledConnection<SqliteConnectionManager>,
) -> Result<(), <Engine as Ingest>::E> {
    for query in [
            format!("DROP TABLE IF EXISTS {COMPLETION_TABLE_NAME};").as_str(),
            format!("DROP TABLE IF EXISTS {EMBEDDINGS_DB_EMBEDDINGS_TABLE_NAME};").as_str(),
            format!("CREATE TABLE IF NOT EXISTS {EMBEDDINGS_DB_EMBEDDINGS_TABLE_NAME} (id INTEGER PRIMARY KEY NOT NULL, gte_small BLOB NOT NULL);").as_str(),
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
