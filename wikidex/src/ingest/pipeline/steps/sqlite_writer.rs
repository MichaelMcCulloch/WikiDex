use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};

use sqlx::SqlitePool;
use tokio::sync::RwLock;

use crate::ingest::pipeline::{
    document::DocumentCompressed,
    error::{PipelineError, Sql},
};

use super::PipelineStep;

pub(crate) struct SqliteWriter {
    pool: Arc<SqlitePool>,
    article_count: Arc<AtomicI64>,
    document_count: Arc<AtomicI64>,
    map: Arc<RwLock<HashMap<String, i64>>>,
}

impl SqliteWriter {
    pub(crate) async fn new(pool: SqlitePool) -> Result<Self, Sql> {
        let mut connection = pool.acquire().await.map_err(Sql::Sql)?;
        let _ = sqlx::query!("BEGIN;",)
            .execute(&mut *connection)
            .await
            .map_err(Sql::Sql)?;
        let _ = sqlx::query!("DROP TABLE IF EXISTS completed_on;",)
            .execute(&mut *connection)
            .await
            .map_err(Sql::Sql)?;
        let _ = sqlx::query!("DROP TABLE IF EXISTS document;",)
            .execute(&mut *connection)
            .await
            .map_err(Sql::Sql)?;
        let _ = sqlx::query!("DROP TABLE IF EXISTS article;",)
            .execute(&mut *connection)
            .await
            .map_err(Sql::Sql)?;
        let _ = sqlx::query!("CREATE TABLE IF NOT EXISTS article ( id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, access_date INTEGER NOT NULL, modification_date INTEGER NOT NULL );",)
            .execute(&mut *connection)
            .await.map_err(Sql::Sql)?;
        let _ = sqlx::query!("CREATE TABLE IF NOT EXISTS document ( id INTEGER PRIMARY KEY NOT NULL,  text BLOB NOT NULL,  article INTEGER NOT NULL,  FOREIGN KEY(article) REFERENCES article(id) );",)
            .execute(&mut *connection)
            .await.map_err(Sql::Sql)?;
        let _ = sqlx::query!("CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER NOT NULL, article_count INTEGER NOT NULL );",)
            .execute(&mut *connection)
            .await.map_err(Sql::Sql)?;
        let _ = sqlx::query!("COMMIT;",)
            .execute(&mut *connection)
            .await
            .map_err(Sql::Sql)?;

        Ok(Self {
            pool: Arc::new(pool),
            article_count: Arc::new(AtomicI64::new(0)),
            document_count: Arc::new(AtomicI64::new(0)),
            map: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}
impl PipelineStep for SqliteWriter {
    type IN = Vec<DocumentCompressed>;

    type OUT = ();

    type ARG = (
        Arc<SqlitePool>,
        Arc<AtomicI64>,
        Arc<AtomicI64>,
        Arc<RwLock<HashMap<String, i64>>>,
    );

    async fn transform(
        documents: Self::IN,
        arg: &Self::ARG,
    ) -> Result<Vec<Self::OUT>, PipelineError> {
        let mut connection = arg.0.acquire().await.map_err(Sql::Sql)?;
        let _ = sqlx::query!("BEGIN TRANSACTION;",)
            .execute(&mut *connection)
            .await;
        for document in documents {
            let access_millis = document.access_date.and_utc().timestamp_millis();
            let modification_millis = document.modification_date.and_utc().timestamp_millis();

            let read_lock = arg.3.read().await;
            let article_id = if let Some(&article_id) = read_lock.get(&document.article_title) {
                // If the article ID is found, return it without requiring a write lock
                article_id
            } else {
                // Acquire a write lock because the key was not found
                drop(read_lock); // Release the read lock before acquiring the write lock
                let mut write_lock = arg.3.write().await;

                // Check again in case another writer added the key while acquiring the lock
                if let Some(&article_id) = write_lock.get(&document.article_title) {
                    article_id
                } else {
                    // Generate a new article ID and update the store
                    let article_id = arg.1.fetch_add(1, Ordering::Relaxed);
                    write_lock.insert(document.article_title.clone(), article_id);

                    // Insert into the database while holding the write lock
                    sqlx::query!(
                        "INSERT INTO article (id, title, access_date, modification_date) VALUES (?1, ?2, ?3, ?4)",
                        article_id,
                        document.article_title,
                        access_millis,
                        modification_millis
                    )
                    .execute(&mut *connection)
                    .await
                    .map_err(Sql::Sql)?;

                    article_id
                }
            };

            let document_id = arg.2.fetch_add(1, Ordering::Relaxed);

            let _rows = sqlx::query!(
                "INSERT INTO document (id, text, article) VALUES (?1, ?2, ?3)",
                document_id,
                document.document,
                article_id
            )
            .execute(&mut *connection)
            .await
            .map_err(Sql::Sql)?;
        }
        let _ = sqlx::query!("COMMIT TRANSACTION;",)
            .execute(&mut *connection)
            .await
            .map_err(Sql::Sql)?;
        Ok(vec![()])
    }

    fn args(&self) -> Self::ARG {
        (
            self.pool.clone(),
            self.article_count.clone(),
            self.document_count.clone(),
            self.map.clone(),
        )
    }
    fn name() -> String {
        String::from("Sqlite Writer")
    }

    async fn link(
        &self,
        mut receiver: tokio::sync::mpsc::UnboundedReceiver<Self::IN>,
        progress: Arc<indicatif::ProgressBar>,
        next_progress: Vec<Arc<indicatif::ProgressBar>>,
    ) -> Result<Vec<tokio::sync::mpsc::UnboundedReceiver<Self::OUT>>, PipelineError> {
        let (sender, new_receiver) = tokio::sync::mpsc::unbounded_channel::<Self::OUT>();
        let args = Arc::new(self.args());
        let next_progress = next_progress
            .first()
            .ok_or(crate::ingest::pipeline::error::LinkError::NoCurrentProgressBar)?
            .clone();

        progress.set_message(Self::name().to_string());
        tokio::spawn(async move {
            while let Some(input) = receiver.recv().await {
                let args = args.clone();
                let sender = sender.clone();
                let progress = progress.clone();
                let next_progress = next_progress.clone();
                tokio::spawn(async move {
                    let transform = Self::transform(input, &args)
                        .await
                        .map_err(PipelineError::from);

                    match transform {
                        Ok(transform) => {
                            progress.inc(1);

                            for _ in transform {
                                next_progress.inc_length(1);

                                sender.send(()).expect("G");
                            }

                            Ok::<(), PipelineError>(())
                        }
                        Err(e) => Err(e),
                    }
                });
            }

            Ok::<(), PipelineError>(())
        });
        Ok(vec![new_receiver])
    }
}
