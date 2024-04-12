use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug)]
pub enum DocstoreLoadError {
    Database(sqlx::error::Error),
    Redis(redis::RedisError),
}
#[derive(Debug)]
pub enum DocstoreRetrieveError {
    IndexOutOfRange,
    InvalidDocument,
    Database(sqlx::error::Error),
    Redis(redis::RedisError),
}

impl std::error::Error for DocstoreLoadError {}
impl std::error::Error for DocstoreRetrieveError {}

impl Display for DocstoreLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DocstoreLoadError::Database(e) => write!(f, "DocstoreLoadError: Database: {e}"),
            DocstoreLoadError::Redis(e) => write!(f, "DocstoreLoadError: Redis: {e}"),
        }
    }
}

impl Display for DocstoreRetrieveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DocstoreRetrieveError::IndexOutOfRange => {
                write!(f, "DocstoreRetrieveError: Index out of range")
            }
            DocstoreRetrieveError::InvalidDocument => {
                write!(f, "DocstoreRetrieveError: Invalid document")
            }
            DocstoreRetrieveError::Database(e) => {
                write!(f, "DocstoreRetrieveError: Database: {e}")
            }
            DocstoreRetrieveError::Redis(e) => {
                write!(f, "DocstoreRetrieveError: Redis: {e}")
            }
        }
    }
}
