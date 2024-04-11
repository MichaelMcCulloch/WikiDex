mod error;

#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

pub(crate) use error::{DocstoreLoadError, DocstoreRetrieveError};

use sqlx::{Database, Pool};

pub(crate) struct Docstore<DB: Database> {
    pool: Pool<DB>,
}
