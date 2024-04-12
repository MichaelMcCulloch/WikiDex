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

use crate::formatter::Provenance;

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
    async fn retreive_from_db(
        &self,
        indices: &[i64],
    ) -> Result<Vec<(usize, String, Provenance)>, DocstoreRetrieveError>;
}
pub(crate) trait DocumentCache: Send + Sync {
    async fn insert_into_cache(
        &self,
        index: i64,
        data: Vec<(usize, String, Provenance)>,
    ) -> Result<(), DocstoreRetrieveError>;
    async fn retreive_from_cache(
        &self,
        indices: &[i64],
    ) -> Result<Vec<(usize, String, Provenance)>, DocstoreRetrieveError>;
}

impl DocumentStore for DocumentStoreKind {
    async fn retreive_from_db(
        &self,
        indices: &[i64],
    ) -> Result<Vec<(usize, String, Provenance)>, DocstoreRetrieveError> {
        match self {
            #[cfg(feature = "postgres")]
            DocumentStoreKind::Postgres(docstore) => docstore.retreive_from_db(indices).await,
            #[cfg(feature = "sqlite")]
            DocumentStoreKind::Sqlite(docstore) => docstore.retreive_from_db(indices).await,
        }
    }
}

impl<DB: Database> DocumentCache for Docstore<DB> {
    async fn retreive_from_cache(
        &self,
        indices: &[i64],
    ) -> Result<Vec<(usize, String, Provenance)>, DocstoreRetrieveError> {
        let mut cache = self.cache.clone();
        let result: Vec<(usize, String, Provenance)> = redis::cmd("MGET")
            .arg(indices)
            .query_async(&mut cache)
            .await
            .map_err(DocstoreRetrieveError::Redis)?;
        Ok(result)
    }
    async fn insert_into_cache(
        &self,
        index: i64,
        data: Vec<(usize, String, Provenance)>,
    ) -> Result<(), DocstoreRetrieveError> {
        let mut cache = self.cache.clone();
        cache
            .set(index, data)
            .await
            .map_err(DocstoreRetrieveError::Redis)?;
        Ok(())
    }
}
