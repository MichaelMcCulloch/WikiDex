use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result},
};

use faiss::error::Error as FsError;

#[derive(Debug)]
pub enum IndexError {
    FileNotFound,
    IndexReadError(FsError),
    IndexFormatError(FsError),
}

#[derive(Debug)]
pub enum IndexSearchError {
    IncorrectDimensions,
}

impl StdError for IndexError {}
impl StdError for IndexSearchError {}

impl Display for IndexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            IndexError::FileNotFound => write!(f, "SearchService: Index not found"),
            IndexError::IndexReadError(err) => {
                write!(f, "SearchService: {}", err)
            }
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
        }
    }
}
