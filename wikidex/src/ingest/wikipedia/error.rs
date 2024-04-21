use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
    io,
    path::PathBuf,
};

use crate::embedding_client::EmbeddingServiceError;

use super::{helper::error::IndexError, markup_processor::WikiMarkupProcessingError};

#[derive(Debug)]
pub(crate) enum IngestError {
    XmlNotFound(PathBuf),
    IoError(io::Error),
    DirectoryNotFound(PathBuf),
    SqlX(sqlx::Error),
    XmlDateReadError,
    EmbeddingServiceError(EmbeddingServiceError),
    Timeout(String),
    MarkupError(WikiMarkupProcessingError),
    NoRows,
    IndexError(IndexError),
    FaissError(faiss::error::Error),
}

impl Error for IngestError {}
impl Display for IngestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            IngestError::XmlNotFound(path) => {
                write!(f, "IngestEngine: Input XML '{}' not found", path.display())
            }
            IngestError::DirectoryNotFound(path) => {
                write!(f, "IngestEngine: Directory '{}' not found", path.display())
            }
            IngestError::IoError(error) => {
                write!(f, "IngestEngine: IO Error: {error}",)
            }
            IngestError::SqlX(error) => {
                write!(f, "IngestEngine: Sqlite Error: {error}",)
            }
            IngestError::XmlDateReadError => {
                write!(f, "IngestEngine: Unable to read data from XML File Name.",)
            }
            IngestError::EmbeddingServiceError(error) => write!(f, "{error}"),
            IngestError::Timeout(s) => {
                write!(f, "IngestEngine: Timeout processing '{s}'")
            }
            IngestError::NoRows => {
                write!(f, "IngestEngine: No rows to process.")
            }
            IngestError::MarkupError(e) => {
                write!(f, "{e}")
            }
            IngestError::IndexError(e) => write!(f, "{e}"),
            IngestError::FaissError(error) => write!(f, "IngestEngine: Faiss Error: {error}",),
        }
    }
}