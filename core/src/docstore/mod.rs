mod error;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

pub(crate) use error::{DocstoreLoadError, DocstoreRetrieveError};

use sqlx::{Database, Pool};

#[cfg(feature = "postgres")]
use sqlx::Postgres;
#[cfg(feature = "sqlite")]
use sqlx::Sqlite;

use crate::formatter::Provenance;

pub(crate) struct Docstore<DB: Database> {
    cache: redis::Client,
    pool: Pool<DB>,
}

pub(crate) enum DocumentStoreKind {
    #[cfg(feature = "postgres")]
    Postgres(Docstore<Postgres>),
    #[cfg(feature = "sqlite")]
    Sqlite(Docstore<Sqlite>),
}

pub(crate) trait DocumentStore: Send + Sync {
    async fn retreive(
        &self,
        indices: &[i64],
    ) -> Result<Vec<(usize, String, Provenance)>, DocstoreRetrieveError>;
}
impl DocumentStore for DocumentStoreKind {
    async fn retreive(
        &self,
        indices: &[i64],
    ) -> Result<Vec<(usize, String, Provenance)>, DocstoreRetrieveError> {
        match self {
            #[cfg(feature = "postgres")]
            DocumentStoreKind::Postgres(docstore) => docstore.retreive(indices).await,
            #[cfg(feature = "sqlite")]
            DocumentStoreKind::Sqlite(docstore) => docstore.retreive(indices).await,
        }
    }
}
