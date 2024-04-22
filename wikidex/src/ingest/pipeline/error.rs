use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result},
};

use crate::ingest::wikipedia::WikiMarkupProcessingError;

#[derive(Debug)]
pub enum WikipediaDumpReaderError {
    XmlDateReadError,
    ErrorReadingDump,
    MarkupError(WikiMarkupProcessingError),
    Timeout(String),
}

impl StdError for WikipediaDumpReaderError {}

impl Display for WikipediaDumpReaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            WikipediaDumpReaderError::XmlDateReadError => {
                write!(f, "WikipediaDumpReaderError::XmlDateReadError")
            }
            WikipediaDumpReaderError::ErrorReadingDump => {
                write!(f, "WikipediaDumpReaderError::ErrorReadingDump")
            }
            WikipediaDumpReaderError::MarkupError(e) => {
                write!(f, "WikipediaDumpReaderError::MarkupError {e}")
            }
            WikipediaDumpReaderError::Timeout(e) => {
                write!(f, "WikipediaDumpReaderError::Timeout {e}")
            }
        }
    }
}

#[derive(Debug)]
pub enum PipelineError {
    QueryError,
    WikipediaDumpReaderError(WikipediaDumpReaderError),
}

impl StdError for PipelineError {}

impl Display for PipelineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PipelineError::QueryError => {
                write!(f, "PipelineError")
            }
            PipelineError::WikipediaDumpReaderError(e) => {
                write!(f, "{e}")
            }
        }
    }
}

impl From<WikipediaDumpReaderError> for PipelineError {
    fn from(value: WikipediaDumpReaderError) -> Self {
        Self::WikipediaDumpReaderError(value)
    }
}
