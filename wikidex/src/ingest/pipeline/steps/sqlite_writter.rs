use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};

use sqlx::SqlitePool;

use crate::ingest::pipeline::document::CompressedDocument;

use super::PipelineStep;

pub(crate) struct SqliteWriter {
    pool: Arc<SqlitePool>,
    article_count: Arc<AtomicI64>,
    document_count: Arc<AtomicI64>,
}

impl SqliteWriter {
    pub(crate) async fn new(pool: SqlitePool) -> Self {
        let mut connection = pool.acquire().await.unwrap();

        let _rows = sqlx::query!("DROP TABLE IF EXISTS completed_on;",)
            .execute(&mut *connection)
            .await;
        let _rows = sqlx::query!("DROP TABLE IF EXISTS document;",)
            .execute(&mut *connection)
            .await;
        let _rows = sqlx::query!("DROP TABLE IF EXISTS article;",)
            .execute(&mut *connection)
            .await;
        let _rows = sqlx::query!("CREATE TABLE IF NOT EXISTS article ( id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, access_date INTEGER NOT NULL, modification_date INTEGER NOT NULL );",)
        .execute(&mut *connection)
        .await;
        let _rows = sqlx::query!("CREATE TABLE IF NOT EXISTS document ( id INTEGER PRIMARY KEY NOT NULL,  text BLOB NOT NULL,  article INTEGER NOT NULL,  FOREIGN KEY(article) REFERENCES article(id) );",)
        .execute(&mut *connection)
        .await;
        let _rows = sqlx::query!("CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER NOT NULL, article_count INTEGER NOT NULL );",)
        .execute(&mut *connection)
        .await;

        Self {
            pool: Arc::new(pool),
            article_count: Arc::new(AtomicI64::new(0)),
            document_count: Arc::new(AtomicI64::new(0)),
        }
    }
}
impl PipelineStep for SqliteWriter {
    type IN = CompressedDocument;

    type OUT = ();

    type ARG = (Arc<SqlitePool>, Arc<AtomicI64>, Arc<AtomicI64>);

    async fn transform(input: Self::IN, arg: &Self::ARG) -> Vec<Self::OUT> {
        let CompressedDocument {
            document,
            article_title,
            access_date,
            modification_date,
        } = input;

        let mut connection = arg.0.acquire().await.unwrap();
        let article_id = arg.1.fetch_add(1, Ordering::Relaxed);
        let access_millis = access_date.and_utc().timestamp_millis();
        let modification_millis = modification_date.and_utc().timestamp_millis();
        let _rows = sqlx::query!(
                        "INSERT INTO article (id, title, access_date, modification_date) VALUES (?1, ?2, ?3, ?4)",
                        article_id,
                        article_title,
                        access_millis,
                        modification_millis
                    )
                    .execute(&mut *connection)
                    .await.unwrap();

        let document_id = arg.2.fetch_add(1, Ordering::Relaxed);

        let _rows = sqlx::query!(
            "INSERT INTO document (id, text, article) VALUES (?1, ?2, ?3)",
            document_id,
            document,
            article_id
        )
        .execute(&mut *connection)
        .await
        .unwrap();

        vec![()]
    }

    fn args(&self) -> Self::ARG {
        (
            self.pool.clone(),
            self.article_count.clone(),
            self.document_count.clone(),
        )
    }
}
