mod cache;
mod database;
mod document;
mod error;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

use self::document::Document;

pub(super) use error::{DocstoreLoadError, DocstoreRetrieveError};
use redis::aio::MultiplexedConnection;
use sqlx::{Database, Pool};

#[cfg(feature = "postgres")]
use sqlx::Postgres;
#[cfg(feature = "sqlite")]
use sqlx::Sqlite;

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
