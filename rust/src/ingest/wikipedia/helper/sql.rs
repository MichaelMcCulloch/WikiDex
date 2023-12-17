use super::{
    super::IngestError::*,
    gzip_helper::decompress_text,
    wiki::{CompressedPage, CompressedPageWithAccessDate, DocumentFragments},
};
use crate::{
    ingest::wikipedia::IngestError,
    openai::OpenAiDelegate,
};
use chrono::NaiveDateTime; 
use indicatif::ProgressBar;
use sqlx::{SqliteConnection, SqlitePool}; 
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use std::{
    io::Write,
    path::Path,
    sync::{
        atomic::{AtomicI64, Ordering}, Arc,
    },
};

pub(crate) async fn write_completion_timestamp(
    pool: &SqlitePool,
    article_count: i64,
) -> Result<(), IngestError> {
    let naive_utc = chrono::Utc::now().naive_utc();
    let timestamp = naive_utc.timestamp_millis();

    let _rows = sqlx::query!(
        "INSERT INTO completed_on (db_date, article_count) VALUES (?1, ?2);",
        timestamp,
        article_count
    )
    .execute(pool)
    .await
    .map_err(Sqlite)?;
    // let rows = sqlx::query_with(&query).execute(connection).await.map_err(Sqlite)?;

    Ok(())
}

pub(crate) async fn populate_markup_db(
    pool: &SqlitePool,
    pages_compressed: Vec<CompressedPage>,
    access_date: NaiveDateTime,
    progress_bar: &ProgressBar,
) -> Result<i64, IngestError> {
    progress_bar.set_message("Writing Compressed Markup to DB...");
    let mut connection = pool.acquire().await.map_err(Sqlite)?;
    let _rows = sqlx::query!("BEGIN;",)
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;

    init_markup_sqlite_pool(&mut connection).await?;

    let article_count = AtomicI64::new(0);

    for CompressedPage {
        gzipped_text,
        article_title,
    } in pages_compressed.into_iter()
    {
        let article_id = article_count.fetch_add(1, Ordering::Relaxed);
        let access_millis = access_date.timestamp_millis();
        let _rows = sqlx::query!(
            "INSERT INTO wiki_markup (id, title, text, access_date) VALUES (?1, ?2, ?3, ?4)",
            article_id,
            article_title,
            gzipped_text,
            access_millis
        )
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;

        progress_bar.inc(1);
    }
    let _rows = sqlx::query!("COMMIT;",)
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;
    progress_bar.set_message("Writing Compressed Markup to DB...DONE");
    Ok(article_count.fetch_add(0, Ordering::SeqCst) )
}

pub(crate) async fn populate_docstore_db(
    pool: &SqlitePool,
    pages_compressed: Vec<DocumentFragments>,
    progress_bar: &ProgressBar,
) -> Result<i64, IngestError> {
    progress_bar.set_message("Writing Docstore to DB...");
    let mut connection = pool.acquire().await.map_err(Sqlite)?;
    let _rows = sqlx::query!("BEGIN;",)
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;
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
        let access_millis = access_date.timestamp_millis();
        let modification_millis = modification_date.timestamp_millis();
        let _rows = sqlx::query!(
            "INSERT INTO article (id , title, access_date, modification_date) VALUES (?1, ?2, ?3, ?4)",
            article_id,
            article_title,
            access_millis, 
            modification_millis
        )
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;

        for document in documents {
            let document_id = document_count.fetch_add(1, Ordering::Relaxed);

            let _rows = sqlx::query!(
                "INSERT INTO document (id, text, article) VALUES (?1, ?2, ?3)",
                document_id,
                document,
                article_id
            )
            .execute(&mut *connection)
            .await
            .map_err(Sqlite)?;
        }

        progress_bar.inc(1);
    }
    let _rows = sqlx::query!("COMMIT;",)
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;
    progress_bar.set_message("Writing Docstore to DB...DONE");
    Ok(document_count.fetch_add(0, Ordering::SeqCst))
}

pub(crate) async fn write_vectorstore(
    mut rx: UnboundedReceiver<(i64, Vec<f32>)>,
    pool: Arc<SqlitePool>,
    create_vectors_bar: Arc<ProgressBar>,
) -> Result<(), IngestError> {
    let mut connection = pool.acquire().await.map_err(Sqlite)?;
    init_temp_embedding_sqlite_pool(&mut connection).await?;

    while let Some((index, embedding)) = rx.recv().await {
        let _rows = sqlx::query!("BEGIN;",)
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;

        let mut v8: Vec<u8> = vec![];

        for e in embedding {
            v8.write_all(&e.to_le_bytes()).map_err(IoError)?;
        }



        let _rows = sqlx::query!(
            "INSERT INTO embeddings (id, gte_small) VALUES (?1, ?2)",
            index, v8
        )
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;
     
        create_vectors_bar.inc(1);

        let _rows = sqlx::query!("COMMIT;",)
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;
    }
    Ok(())
}
pub(crate) async fn populate_vectorstore_db(
    openai: &OpenAiDelegate,
    pool: &SqlitePool,
    document_count: i64,
    tx: UnboundedSender<(i64, Vec<f32>)>,
) -> Result<(), IngestError> {
    let mut connection = pool.acquire().await.map_err(Sqlite)?;

    for index in 0..document_count {
        let tx = tx.clone();

        let row = sqlx::query!(
            "SELECT text FROM document WHERE id == ?1;",index
        )
        .fetch_one(&mut *connection)
        .await
        .map_err(Sqlite)?; 

        let bytes = row.text;
        let document = decompress_text(bytes).map_err(IoError)?;

 
        let embedding = openai.embed(&document).await.map_err(EmbeddingServiceError)?;
        let _ = tx.send((index, embedding));
    }

    Ok(())
}
pub(crate) async fn database_is_complete(pool: &SqlitePool) -> Result<bool, IngestError> {
    let mut connection = pool.acquire().await.map_err(Sqlite)?;

    let record = sqlx::query!(
        "SELECT * FROM completed_on;"
    )
    .fetch_one(&mut *connection)
    .await ;

 
    match record {
        Ok(_) =>  Ok( true),
        
        Err(_) => Ok(false),
    }
}

pub(crate) async fn count_elements(
    pool: &SqlitePool,
) -> Result<Option<i64>, IngestError> {
    let mut connection = pool.acquire().await.map_err(Sqlite)?;

    let record = sqlx::query!(
        "SELECT article_count FROM completed_on LIMIT 1;"
    )
    .fetch_one(&mut *connection)
    .await ;

    match record {
        Ok(record) => Ok(Some(record.article_count)),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(e) => Err(Sqlite(e))
    }

}
pub(crate) async fn obtain_markup(
    pool: &SqlitePool,
    progress_bar: &ProgressBar,
) -> Result<Vec<CompressedPageWithAccessDate>, IngestError> {
    progress_bar.set_message("Obtaining Markup...");
    let mut connection = pool.acquire().await.map_err(Sqlite)?;

    let records = sqlx::query!(
        "SELECT * FROM wiki_markup;"
    )
    .map(|record| {
        let gzipped_text = record.text;
        let article_title = record.title;
        let timestamp_millis = record.access_date;
        progress_bar.inc(1);
        CompressedPageWithAccessDate {
            gzipped_text,
            article_title,
            access_date: NaiveDateTime::from_timestamp_millis(timestamp_millis).unwrap(),
        }
    })
    .fetch_all(&mut *connection)
    .await.map_err(Sqlite)?;




    

    progress_bar.set_message("Obtaining Markup...DONE");
    Ok(records)
}
pub(crate) async fn obtain_vectors(
    pool: &SqlitePool,
    progress_bar: &ProgressBar,
) -> Result<Vec<Vec<f32>>, IngestError> {
    progress_bar.set_message("Obtaining vectors...");
    let mut connection = pool.acquire().await.map_err(Sqlite)?;

    let records = sqlx::query!(
        "SELECT gte_small FROM embeddings ORDER BY id ASC;"
    )
    .map(|record| {
        let embedding_bytes = record.gte_small;
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
    .await.map_err(Sqlite)?;

    progress_bar.set_message("Obtaining vectors...DONE");
    
    Ok(records)
}
pub(crate) async fn get_sqlite_pool<P: AsRef<Path>>(path: &P) -> Result<SqlitePool, IngestError> {
    let path = path.as_ref();
    let path = path.to_path_buf();
    let path = path
            .to_str()
            .map(|path| format!("sqlite:/{path}"))
            .ok_or(IngestError::DirectoryNotFound(
                path.to_path_buf(),
            ))?;

    SqlitePool::connect(path.as_ref()).await.map_err(Sqlite)
}

pub(crate) async fn init_markup_sqlite_pool(
    connection: &mut SqliteConnection,
) -> Result<(), IngestError> {

    let _rows = sqlx::query!(
        "DROP TABLE IF EXISTS completed_on;",
    )
    .execute(&mut *connection)
    .await
    .map_err(Sqlite)?;
    let _rows = sqlx::query!(
        "DROP TABLE IF EXISTS wiki_markup;",
    )
    .execute(&mut *connection)
    .await
    .map_err(Sqlite)?;
    let _rows = sqlx::query!(
        "CREATE TABLE IF NOT EXISTS wiki_markup ( id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, text BLOB NOT NULL, access_date INTEGER NOT NULL );",
    )
    .execute(&mut *connection)
    .await
    .map_err(Sqlite)?;
    let _rows = sqlx::query!(
        "CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER NOT NULL, article_count INTEGER NOT NULL );",
    )
    .execute(&mut *connection)
    .await
    .map_err(Sqlite)?;


    Ok(())
}

pub(crate) async fn init_docstore_sqlite_pool(
    connection: &mut SqliteConnection,
) -> Result<(), IngestError> {

let _rows = sqlx::query!(
"DROP TABLE IF EXISTS completed_on;",
)
.execute(&mut *connection)
.await
.map_err(Sqlite)?;
let _rows = sqlx::query!(
"DROP TABLE IF EXISTS document;",
)
.execute(&mut *connection)
.await
.map_err(Sqlite)?;
let _rows = sqlx::query!(
"DROP TABLE IF EXISTS article;",
)
.execute(&mut *connection)
.await
.map_err(Sqlite)?;
let _rows = sqlx::query!(
"CREATE TABLE IF NOT EXISTS article ( id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, access_date INTEGER NOT NULL, modification_date INTEGER NOT NULL );",
)
.execute(&mut *connection)
.await
.map_err(Sqlite)?;
let _rows = sqlx::query!(
"CREATE TABLE IF NOT EXISTS document ( id INTEGER PRIMARY KEY NOT NULL,  text BLOB NOT NULL,  article INTEGER NOT NULL,  FOREIGN KEY(article) REFERENCES article(id) );",
)
.execute(&mut *connection)
.await
.map_err(Sqlite)?;
let _rows = sqlx::query!(
"CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER NOT NULL, article_count INTEGER NOT NULL );",
)
.execute(&mut *connection)
.await
.map_err(Sqlite)?;



    Ok(())
}
pub(crate) async fn init_temp_embedding_sqlite_pool(
    connection: &mut SqliteConnection,
) -> Result<(), IngestError> {


    let _rows = sqlx::query!(
        "DROP TABLE IF EXISTS completed_on;",
        )
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;
    let _rows = sqlx::query!(
        "DROP TABLE IF EXISTS embeddings;",
        )
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;
    let _rows = sqlx::query!(
        "CREATE TABLE IF NOT EXISTS embeddings ( id INTEGER PRIMARY KEY NOT NULL, gte_small BLOB NOT NULL);",
        )
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;
    let _rows = sqlx::query!(
        "CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER NOT NULL, article_count INTEGER NOT NULL);",
        )
        .execute(&mut *connection)
        .await
        .map_err(Sqlite)?;
        



    Ok(())
}

