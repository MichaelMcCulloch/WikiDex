use faiss::error::Error as FsError;
use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result},
};

#[derive(Debug)]
pub enum IndexLoadError {
    FileNotFound,
    IndexReadError(FsError),
    IndexFormatError(FsError),
}

#[derive(Debug)]
pub enum IndexSearchError {
    IncorrectDimensions,
    IndexSearchError(FsError),
}

impl StdError for IndexLoadError {}
impl StdError for IndexSearchError {}

impl Display for IndexLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            IndexLoadError::FileNotFound => write!(f, "SearchService: Index not found"),
            IndexLoadError::IndexReadError(err) => {
                write!(f, "SearchService: {}", err)
            }
            IndexLoadError::IndexFormatError(err) => {
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
            IndexSearchError::IndexSearchError(err) => {
                write!(f, "SearchService: {}", err)
            }
        }
    }
}
