use super::{
    super::IngestError::*,
    gzip_helper::decompress_text,
    wiki::{CompressedPage, CompressedPageWithAccessDate, DocumentFragments},
};
use crate::{ingest::wikipedia::IngestError, openai::OpenAiDelegate};

use chrono::{DateTime, NaiveDateTime};
use futures::TryFutureExt;
use indicatif::ProgressBar;
use sqlx::{migrate::MigrateDatabase, PgConnection, PgPool};
use std::{
    io::Write,
    path::Path,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

const BATCH_SIZE: usize = 2048;

pub(crate) async fn write_completion_timestamp(
    pool: &PgPool,
    article_count: i64,
) -> Result<(), IngestError> {
    let naive_utc = chrono::Utc::now().naive_utc();
    let timestamp = naive_utc.and_utc().timestamp_millis();

    let _rows = sqlx::query!(
        "INSERT INTO completed_on (db_date, article_count) VALUES ($1, $2);",
        timestamp,
        article_count
    )
    .execute(pool)
    .await
    .map_err(SqlX)?;

    Ok(())
}

pub(crate) async fn populate_markup_db(
    pool: &PgPool,
    pages_compressed: Vec<CompressedPage>,
    access_date: NaiveDateTime,
    progress_bar: &ProgressBar,
) -> Result<i64, IngestError> {
    progress_bar.set_message("Writing Compressed Markup to DB...");
    let mut connection = pool.acquire().await.map_err(SqlX)?;
    let _rows = sqlx::query!("BEGIN;",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;

    init_markup_sqlite_pool(&mut connection).await?;

    let article_count = AtomicI64::new(0);

    for CompressedPage {
        gzipped_text,
        article_title,
    } in pages_compressed.into_iter()
    {
        let article_id = article_count.fetch_add(1, Ordering::Relaxed);
        let access_millis = access_date.and_utc().timestamp_millis();
        let _rows = sqlx::query!(
            "INSERT INTO wiki_markup (id, title, text, access_date) VALUES ($1, $2, $3, $4)",
            article_id,
            article_title,
            gzipped_text,
            access_millis
        )
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;

        progress_bar.inc(1);
    }
    let _rows = sqlx::query!("COMMIT;",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    progress_bar.set_message("Writing Compressed Markup to DB...DONE");
    Ok(article_count.fetch_add(0, Ordering::SeqCst))
}

pub(crate) async fn populate_docstore_db(
    pool: &PgPool,
    pages_compressed: Vec<DocumentFragments>,
    progress_bar: &ProgressBar,
) -> Result<i64, IngestError> {
    progress_bar.set_message("Writing Docstore to DB...");
    let mut connection = pool.acquire().await.map_err(SqlX)?;
    let _rows = sqlx::query!("BEGIN;",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    init_docstore_sqlite_pool(&mut connection).await?;

    let article_count = AtomicI64::new(0);
    let document_count = AtomicI64::new(0);

    for DocumentFragments {
        documents,
        article_title,
        access_date,
        modification_date,
    } in pages_compressed.into_iter()
    {
        let article_id = article_count.fetch_add(1, Ordering::Relaxed);
        let access_millis = access_date.and_utc().timestamp_millis();
        let modification_millis = modification_date.and_utc().timestamp_millis();
        let _rows = sqlx::query!(
            "INSERT INTO article (id , title, access_date, modification_date) VALUES ($1, $2, $3, $4)",
            article_id,
            article_title,
            access_millis,
            modification_millis
        )
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;

        for document in documents {
            let document_id = document_count.fetch_add(1, Ordering::Relaxed);

            let _rows = sqlx::query!(
                "INSERT INTO document (id, text, article) VALUES ($1, $2, $3)",
                document_id,
                document,
                article_id
            )
            .execute(&mut *connection)
            .await
            .map_err(SqlX)?;
        }

        progress_bar.inc(1);
    }
    let _rows = sqlx::query!("COMMIT;",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    progress_bar.set_message("Writing Docstore to DB...DONE");
    Ok(document_count.fetch_add(0, Ordering::SeqCst))
}

pub(crate) async fn write_vectorstore(
    mut rx: UnboundedReceiver<Vec<(i64, Vec<f32>)>>,
    pool: Arc<PgPool>,
    create_vectors_bar: Arc<ProgressBar>,
) -> Result<(), IngestError> {
    let mut connection = pool.acquire().await.map_err(SqlX)?;
    init_temp_embedding_sqlite_pool(&mut connection).await?;

    while let Some(embeddings) = rx.recv().await {
        let _rows = sqlx::query!("BEGIN;",)
            .execute(&mut *connection)
            .await
            .map_err(SqlX)?;
        for (index, embedding) in embeddings {
            let mut v8: Vec<u8> = vec![];

            for e in embedding {
                v8.write_all(&e.to_le_bytes()).map_err(IoError)?;
            }

            let _rows = sqlx::query!(
                "INSERT INTO embeddings (id, gte_small) VALUES ($1, $2)",
                index,
                v8
            )
            .execute(&mut *connection)
            .await
            .map_err(SqlX)?;
            create_vectors_bar.inc(1);
        }

        let _rows = sqlx::query!("COMMIT;",)
            .execute(&mut *connection)
            .await
            .map_err(SqlX)?;
    }
    Ok(())
}
pub(crate) async fn populate_vectorstore_db(
    openai: Arc<OpenAiDelegate>,
    pool: &PgPool,
    document_count: i64,
    tx: UnboundedSender<Vec<(i64, Vec<f32>)>>,
) -> Result<(), IngestError> {
    let openai = Arc::new(openai);

    /// TODO: https://github.com/michaelfeil/infinity/issues/114#issuecomment-1965382083
    // retry(ExponentialBackoff::default(), || async {
    //     Ok(openai.embed_up().await?)
    // })
    // .await
    // .unwrap();
    for indices in (0..document_count).step_by(BATCH_SIZE) {
        let tx = tx.clone();
        let openai = openai.clone();
        let mut connection = pool.acquire().await.map_err(SqlX)?;
        let start = indices;
        let end = indices + BATCH_SIZE as i64;

        actix_web::rt::spawn(async move {
            let texts = sqlx::query!(
                "SELECT text FROM document WHERE id >= $1 AND id < $2 ORDER BY id ASC;",
                start,
                end
            )
            .map(|row| row.text)
            .fetch_all(&mut *connection)
            .await
            .map_err(SqlX)?
            .into_iter()
            .filter_map(|record| record.and_then(|record| decompress_text(record).ok()))
            .collect::<Vec<_>>();

            match openai.embed_batch(texts).await {
                Ok(embeddings) => {
                    let _ = tx.send(
                        (indices..(indices + BATCH_SIZE as i64))
                            .zip(embeddings)
                            .collect::<Vec<_>>(),
                    );
                }
                Err(e) => {
                    log::error!("{e}")
                }
            }

            Ok::<(), IngestError>(())
        });
    }

    Ok(())
}
pub(crate) async fn database_is_complete(pool: &PgPool) -> Result<bool, IngestError> {
    let mut connection = pool.acquire().await.map_err(SqlX)?;

    let record = sqlx::query!("SELECT * FROM completed_on;")
        .fetch_one(&mut *connection)
        .await;

    match record {
        Ok(_) => Ok(true),

        Err(_) => Ok(false),
    }
}

pub(crate) async fn count_elements(pool: &PgPool) -> Result<Option<i64>, IngestError> {
    let mut connection = pool.acquire().await.map_err(SqlX)?;

    let record = sqlx::query!("SELECT article_count FROM completed_on LIMIT 1;")
        .fetch_one(&mut *connection)
        .await;
    match record {
        Ok(record) => Ok(record.article_count.or(None)),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(e) => Err(SqlX(e)),
    }
}
pub(crate) async fn obtain_markup(
    pool: &PgPool,
    progress_bar: &ProgressBar,
) -> Result<Vec<CompressedPageWithAccessDate>, IngestError> {
    progress_bar.set_message("Obtaining Markup...");
    let mut connection = pool.acquire().await.map_err(SqlX)?;

    let records = sqlx::query!("SELECT * FROM wiki_markup;")
        .map(|record| {
            let gzipped_text = record.text.unwrap();
            let article_title = record.title.unwrap();
            let timestamp_millis = record.access_date.unwrap();
            progress_bar.inc(1);
            CompressedPageWithAccessDate {
                gzipped_text,
                article_title,
                access_date: DateTime::from_timestamp_millis(timestamp_millis)
                    .unwrap()
                    .naive_utc(),
            }
        })
        .fetch_all(&mut *connection)
        .await
        .map_err(SqlX)?;

    progress_bar.set_message("Obtaining Markup...DONE");
    Ok(records)
}
pub(crate) async fn obtain_vectors(
    pool: &PgPool,
    progress_bar: &ProgressBar,
) -> Result<Vec<Vec<f32>>, IngestError> {
    progress_bar.set_message("Obtaining vectors...");
    let mut connection = pool.acquire().await.map_err(SqlX)?;

    let records = sqlx::query!("SELECT gte_small FROM embeddings ORDER BY id ASC;")
        .map(|record| {
            let embedding_bytes = record.gte_small.unwrap();
            let mut embedding: Vec<f32> = vec![];
            for f32_bytes in embedding_bytes.chunks_exact(4) {
                let mut b = [0u8; 4];
                b.copy_from_slice(f32_bytes);
                embedding.push(f32::from_le_bytes(b));
            }
            progress_bar.inc(1);
            embedding
        })
        .fetch_all(&mut *connection)
        .await
        .map_err(SqlX)?;

    progress_bar.set_message("Obtaining vectors...DONE");

    Ok(records)
}
pub(crate) async fn get_sqlite_pool<P: AsRef<Path>>(path: &P) -> Result<PgPool, IngestError> {
    let path = path.as_ref();
    let path = path.to_path_buf();
    let path = path
        .to_str()
        .map(|path| path.to_string())
        .ok_or(IngestError::DirectoryNotFound(path.to_path_buf()))?;

    if !sqlx::Postgres::database_exists(&path).await.map_err(SqlX)? {
        sqlx::Postgres::create_database(&path).await.map_err(SqlX)?;
    }

    PgPool::connect(&path).await.map_err(SqlX)
}

pub(crate) async fn init_markup_sqlite_pool(
    connection: &mut PgConnection,
) -> Result<(), IngestError> {
    let _rows = sqlx::query!("DROP TABLE IF EXISTS completed_on;",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    let _rows = sqlx::query!("DROP TABLE IF EXISTS wiki_markup;",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    let _rows = sqlx::query!("CREATE TABLE IF NOT EXISTS wiki_markup ( id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, text BLOB NOT NULL, access_date INTEGER NOT NULL );",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    let _rows = sqlx::query!("CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER NOT NULL, article_count INTEGER NOT NULL );",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;

    Ok(())
}

pub(crate) async fn init_docstore_sqlite_pool(
    connection: &mut PgConnection,
) -> Result<(), IngestError> {
    let _rows = sqlx::query!("DROP TABLE IF EXISTS completed_on;",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    let _rows = sqlx::query!("DROP TABLE IF EXISTS document;",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    let _rows = sqlx::query!("DROP TABLE IF EXISTS article;",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    let _rows = sqlx::query!("CREATE TABLE IF NOT EXISTS article ( id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, access_date INTEGER NOT NULL, modification_date INTEGER NOT NULL );",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    let _rows = sqlx::query!("CREATE TABLE IF NOT EXISTS document ( id INTEGER PRIMARY KEY NOT NULL,  text BLOB NOT NULL,  article INTEGER NOT NULL,  FOREIGN KEY(article) REFERENCES article(id) );",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    let _rows = sqlx::query!("CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER NOT NULL, article_count INTEGER NOT NULL );",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;

    Ok(())
}
pub(crate) async fn init_temp_embedding_sqlite_pool(
    connection: &mut PgConnection,
) -> Result<(), IngestError> {
    let _rows = sqlx::query!("DROP TABLE IF EXISTS completed_on;",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    let _rows = sqlx::query!("DROP TABLE IF EXISTS embeddings;",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    let _rows = sqlx::query!("CREATE TABLE IF NOT EXISTS embeddings ( id INTEGER PRIMARY KEY NOT NULL, gte_small BLOB NOT NULL);",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;
    let _rows = sqlx::query!("CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER NOT NULL, article_count INTEGER NOT NULL);",)
        .execute(&mut *connection)
        .await
        .map_err(SqlX)?;

    Ok(())
}
