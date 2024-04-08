use face_api::apis::{crate_api::QueryError, Error};
use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result},
};

#[cfg(feature = "ingest")]
use faiss::error::Error as FsError;

#[derive(Debug)]
pub enum IndexError {
    FileNotFound,
    #[cfg(feature = "ingest")]
    IndexReadError(FsError),
    #[cfg(feature = "ingest")]
    IndexFormatError(FsError),
}

#[derive(Debug)]
pub enum IndexSearchError {
    IncorrectDimensions,
    QueryError(Error<QueryError>),
}

impl StdError for IndexError {}
impl StdError for IndexSearchError {}

impl Display for IndexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            IndexError::FileNotFound => write!(f, "SearchService: Index not found"),
            #[cfg(feature = "ingest")]
            IndexError::IndexReadError(err) => {
                write!(f, "SearchService: {}", err)
            }
            #[cfg(feature = "ingest")]
            IndexError::IndexFormatError(err) => {
                write!(f, "SearchService: {}", err)
            }
        }
    }
}

impl Display for IndexSearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            IndexSearchError::IncorrectDimensions => {
                write!(f, "SearchService: Incorrect dimensions for search")
            }
            IndexSearchError::QueryError(err) => {
                write!(f, "SearchService: {:?}", err)
            }
        }
    }
}
