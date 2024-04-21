use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum IndexError {
    FileNotFound,
    IndexReadError(faiss::error::Error),
    IndexFormatError(faiss::error::Error),
}

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
