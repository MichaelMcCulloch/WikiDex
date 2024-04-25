use std::{
    sync::{
        Arc,
    },
};

use sqlx::SqlitePool;



use crate::ingest::pipeline::{
    document::DocumentCompressed,
    error::{PipelineError, Sql},
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
            docstore_pool: Arc::new(docstore_pool),
            index_pool: Arc::new(index_pool),
        })
    }
}
impl PipelineStep for SqliteWriter {
    type IN = Vec<DocumentCompressed>;

    type OUT = ();

    type ARG = Arc<SqlitePool>;

    async fn transform(
        documents: Self::IN,
        arg: &Self::ARG,
    ) -> Result<Vec<Self::OUT>, PipelineError> {
        let mut connection = arg.acquire().await.map_err(Sql::Sql)?;
        let _ = sqlx::query!("BEGIN TRANSACTION;",)
            .execute(&mut *connection)
            .await;
        for document in documents {
            let access_millis = document.access_date.and_utc().timestamp_millis();
            let modification_millis = document.modification_date.and_utc().timestamp_millis();

            sqlx::query!(
                "INSERT INTO article (id, title, access_date, modification_date) VALUES (?1, ?2, ?3, ?4)",
                document.article_id,
                document.article_title,
                access_millis,
                modification_millis
            )
            .execute(&mut *connection)
            .await
            .map_err(Sql::Sql)?;

            let _rows = sqlx::query!(
                "INSERT INTO document (id, text, article) VALUES (?1, ?2, ?3)",
                document.document_id,
                document.document,
                document.article_id
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
        self.docstore_pool.clone()
    }
    fn name() -> String {
        String::from("Sqlite Writer")
    }
}
