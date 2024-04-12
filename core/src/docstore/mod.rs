mod document;
mod error;

#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

pub(crate) use error::{DocstoreLoadError, DocstoreRetrieveError};

use redis::{aio::MultiplexedConnection, AsyncCommands};
use sqlx::{Database, Pool};

#[cfg(feature = "postgres")]
use sqlx::Postgres;
#[cfg(feature = "sqlite")]
use sqlx::Sqlite;



use self::document::Document;

pub(crate) struct Docstore<DB: Database> {
    cache: MultiplexedConnection,
    pool: Pool<DB>,
}

pub(crate) enum DocumentStoreKind {
    #[cfg(feature = "postgres")]
    Postgres(Docstore<Postgres>),
    #[cfg(feature = "sqlite")]
    Sqlite(Docstore<Sqlite>),
}

pub(crate) trait DocumentStore: Send + Sync {
    async fn retreive(&self, indices: &[i64]) -> Result<Vec<Document>, DocstoreRetrieveError>;
}
pub(crate) trait DocumentDatabase: Send + Sync {
    async fn retreive_from_db(
        &self,
        indices: &[i64],
    ) -> Result<Vec<Document>, DocstoreRetrieveError>;
}
pub(crate) trait DocumentCache: Send + Sync {
    async fn insert_into_cache(
        &self,
        index: i64,
        data: Vec<Document>,
    ) -> Result<(), DocstoreRetrieveError>;
    async fn retreive_from_cache(
        &self,
        indices: &[i64],
    ) -> Result<Vec<Document>, DocstoreRetrieveError>;
}

impl DocumentStore for DocumentStoreKind {
    async fn retreive(&self, indices: &[i64]) -> Result<Vec<Document>, DocstoreRetrieveError> {
        match self {
            #[cfg(feature = "postgres")]
            DocumentStoreKind::Postgres(docstore) => docstore.retreive(indices).await,
            #[cfg(feature = "sqlite")]
            DocumentStoreKind::Sqlite(docstore) => docstore.retreive(indices).await,
        }
    }
}

impl<DB: Database> DocumentCache for Docstore<DB> {
    async fn retreive_from_cache(
        &self,
        indices: &[i64],
    ) -> Result<Vec<Document>, DocstoreRetrieveError> {
        let mut cache = self.cache.clone();
        let result: Vec<Document> = redis::cmd("MGET")
            .arg(indices)
            .query_async(&mut cache)
            .await
            .map_err(DocstoreRetrieveError::Redis)?;
        Ok(result)
    }
    async fn insert_into_cache(
        &self,
        index: i64,
        data: Vec<Document>,
    ) -> Result<(), DocstoreRetrieveError> {
        let mut cache = self.cache.clone();
        cache
            .set(index, data)
            .await
            .map_err(DocstoreRetrieveError::Redis)?;
        Ok(())
    }
}
impl<DB: Database> DocumentStore for Docstore<DB> {
    async fn retreive(&self, indices: &[i64]) -> Result<Vec<Document>, DocstoreRetrieveError> {
        let _cached_documents = self.retreive_from_cache(indices).await?;
        Ok(vec![])
    }
}
