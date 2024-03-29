use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug)]
pub enum DocstoreLoadError {
    FileNotFound,
}
#[derive(Debug)]
pub enum DocstoreRetrieveError {
    IndexOutOfRange,
    InvalidDocument,
    SqlxError(sqlx::error::Error),
}

impl std::error::Error for DocstoreLoadError {}
impl std::error::Error for DocstoreRetrieveError {}

impl Display for DocstoreLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DocstoreLoadError::FileNotFound => write!(f, "DocumentService: File not found"),
        }
    }
}

impl Display for DocstoreRetrieveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DocstoreRetrieveError::IndexOutOfRange => {
                write!(f, "DocumentService: Index out of range")
            }
            DocstoreRetrieveError::InvalidDocument => {
                write!(f, "DocumentService: Invalid document")
            }
            DocstoreRetrieveError::SqlxError(e) => {
                write!(f, "DocumentService: {e}")
            }
        }
    }
}
