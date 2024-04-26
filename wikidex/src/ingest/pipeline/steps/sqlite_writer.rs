use std::{io::Write, sync::Arc};

use indicatif::ProgressBar;
use sqlx::{Pool, Sqlite, SqlitePool};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use crate::ingest::pipeline::{
    document::DocumentCompressed,
    error::{LinkError, PipelineError, Sql},
};

use super::PipelineStep;

pub(crate) struct SqliteWriter {
    docstore_pool: Arc<SqlitePool>,
    index_pool: Arc<SqlitePool>,
}

impl SqliteWriter {
    pub(crate) async fn new(
        docstore_pool: SqlitePool,
        index_pool: SqlitePool,
    ) -> Result<Self, Sql> {
        create_docstore_schemas(&docstore_pool).await?;
        create_index_schemas(&index_pool).await?;

        Ok(Self {
            docstore_pool: Arc::new(docstore_pool),
            index_pool: Arc::new(index_pool),
        })
    }
}

async fn create_docstore_schemas(docstore_pool: &Pool<Sqlite>) -> Result<(), Sql> {
    let mut connection = docstore_pool.acquire().await.map_err(Sql::Sql)?;
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
    let _ = sqlx::query!("CREATE TABLE IF NOT EXISTS article ( id INTEGER PRIMARY KEY NOT NULL, title TEXT NOT NULL, access_date INTEGER NOT NULL, modification_date INTEGER NOT NULL, unique(id) );",)
        .execute(&mut *connection)
        .await.map_err(Sql::Sql)?;
    let _ = sqlx::query!("CREATE TABLE IF NOT EXISTS document ( id INTEGER PRIMARY KEY NOT NULL,  text BLOB NOT NULL,  article INTEGER NOT NULL,  FOREIGN KEY(article) REFERENCES article(id), unique(id) );",)
        .execute(&mut *connection)
        .await.map_err(Sql::Sql)?;
    let _ = sqlx::query!("CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER NOT NULL, article_count INTEGER NOT NULL );",)
        .execute(&mut *connection)
        .await.map_err(Sql::Sql)?;
    let _ = sqlx::query!("COMMIT;",)
        .execute(&mut *connection)
        .await
        .map_err(Sql::Sql)?;
    Ok(())
}

async fn create_index_schemas(docstore_pool: &Pool<Sqlite>) -> Result<(), Sql> {
    let mut connection = docstore_pool.acquire().await.map_err(Sql::Sql)?;
    let _ = sqlx::query!("BEGIN;",)
        .execute(&mut *connection)
        .await
        .map_err(Sql::Sql)?;
    let _ = sqlx::query!("DROP TABLE IF EXISTS completed_on;",)
        .execute(&mut *connection)
        .await
        .map_err(Sql::Sql)?;
    let _ = sqlx::query!("DROP TABLE IF EXISTS embeddings;",)
        .execute(&mut *connection)
        .await
        .map_err(Sql::Sql)?;
    let _ = sqlx::query!("CREATE TABLE IF NOT EXISTS embeddings ( id INTEGER PRIMARY KEY NOT NULL, gte_small BLOB NOT NULL, unique(id));",)
        .execute(&mut *connection)
        .await.map_err(Sql::Sql)?;
    let _ = sqlx::query!("CREATE TABLE IF NOT EXISTS completed_on ( db_date INTEGER NOT NULL, article_count INTEGER NOT NULL );",)
        .execute(&mut *connection)
        .await.map_err(Sql::Sql)?;
    let _ = sqlx::query!("COMMIT;",)
        .execute(&mut *connection)
        .await
        .map_err(Sql::Sql)?;
    Ok(())
}
impl PipelineStep for SqliteWriter {
    type IN = Vec<DocumentCompressed>;
    type OUT = ();
    type ARG = (Arc<SqlitePool>, Arc<SqlitePool>);

    async fn transform(
        documents: Self::IN,
        pools: &Self::ARG,
    ) -> Result<Vec<Self::OUT>, PipelineError> {
        let mut docstore_connection = pools.0.acquire().await.map_err(Sql::Sql)?;
        let mut index_connection = pools.1.acquire().await.map_err(Sql::Sql)?;
        let _ = sqlx::query!("BEGIN TRANSACTION;",)
            .execute(&mut *docstore_connection)
            .await;
        let _ = sqlx::query!("BEGIN TRANSACTION;",)
            .execute(&mut *index_connection)
            .await;
        for document in documents {
            let access_millis = document.access_date.and_utc().timestamp_millis();
            let modification_millis = document.modification_date.and_utc().timestamp_millis();
            let document_embedding = {
                let mut v8: Vec<u8> = vec![];
                for e in document.embedding {
                    v8.write_all(&e.to_le_bytes()).unwrap();
                }
                v8
            };
            let _article_future = sqlx::query!(
                "INSERT OR IGNORE INTO article (id, title, access_date, modification_date) VALUES (?1, ?2, ?3, ?4)",
                document.article_id,
                document.article_title,
                access_millis,
                modification_millis
            )
            .execute(&mut *docstore_connection)
            .await
            .map_err(Sql::Sql)?;

            let _document_future = sqlx::query!(
                "INSERT INTO document (id, text, article) VALUES (?1, ?2, ?3)",
                document.document_id,
                document.document,
                document.article_id
            )
            .execute(&mut *docstore_connection)
            .await
            .map_err(Sql::Sql)?;

            let _emebedding_future = sqlx::query!(
                "INSERT INTO embeddings (id, gte_small) VALUES (?1, ?2)",
                document.document_id,
                document_embedding,
            )
            .execute(&mut *index_connection)
            .await
            .map_err(Sql::Sql)?;
        }
        let _ = sqlx::query!("COMMIT TRANSACTION;",)
            .execute(&mut *docstore_connection)
            .await
            .map_err(Sql::Sql)?;
        let _ = sqlx::query!("COMMIT TRANSACTION;",)
            .execute(&mut *index_connection)
            .await
            .map_err(Sql::Sql)?;
        Ok(vec![()])
    }

    fn args(&self) -> Self::ARG {
        (self.docstore_pool.clone(), self.index_pool.clone())
    }
    fn name() -> String {
        String::from("Sqlite Writer")
    }

    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
        progress: Arc<ProgressBar>,
        next_progress: Vec<Arc<ProgressBar>>,
    ) -> Result<Vec<UnboundedReceiver<Self::OUT>>, PipelineError> {
        let (sender, new_receiver) = unbounded_channel::<Self::OUT>();
        let args = Arc::new(self.args());
        let next_progress = next_progress
            .first()
            .ok_or(LinkError::NoCurrentProgressBar(Self::name()))?
            .clone();

        progress.set_message(Self::name().to_string());
        tokio::spawn(async move {
            while let Some(input) = receiver.recv().await {
                let len = input.len() as u64;
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
                            progress.inc(len);

                            for t in transform {
                                next_progress.inc_length(1);

                                let _ = sender.send(t);
                            }
                        }
                        Err(e) => {
                            log::warn!("{} {e}", Self::name())
                        }
                    }

                    Ok::<(), PipelineError>(())
                });
            }

            Ok::<(), PipelineError>(())
        });
        Ok(vec![new_receiver])
    }
}
