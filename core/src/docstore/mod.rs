mod error;

mod postgres;
mod sqlite;

pub(crate) use error::{DocstoreLoadError, DocstoreRetrieveError};

use sqlx::{Database, Pool, Postgres, Sqlite};

use crate::formatter::Provenance;

pub(crate) struct Docstore<DB: Database> {
    pool: Pool<DB>,
}

pub(crate) enum DocumentStoreKind {
    Postgres(Docstore<Postgres>),
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
            DocumentStoreKind::Postgres(docstore) => docstore.retreive(indices).await,
            DocumentStoreKind::Sqlite(docstore) => docstore.retreive(indices).await,
        }
    }
}
