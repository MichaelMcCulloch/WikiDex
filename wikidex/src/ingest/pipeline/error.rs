use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result},
    io,
};

use tokio::sync::oneshot;

use crate::embedding_client::EmbeddingServiceError;

use super::wikipedia::WikiMarkupProcessingError;

#[derive(Debug)]
pub enum PipelineError {
    QueryError,
    LinkError(LinkError),
    EmbeddingError(EmbeddingError),
    CompressionError(CompressionError),
    WikipediaDumpReaderError(WikipediaDumpReaderError),
    ParseError(ParseError),
    Sql(Sql),
}
impl StdError for PipelineError {}
impl Display for PipelineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PipelineError::QueryError => write!(f, "PipelineError"),
            PipelineError::WikipediaDumpReaderError(e) => write!(f, "{e}"),
            PipelineError::Sql(e) => write!(f, "{e}"),
            PipelineError::LinkError(e) => write!(f, "{e}"),
            PipelineError::EmbeddingError(e) => write!(f, "{e}"),
            PipelineError::CompressionError(e) => write!(f, "{e}"),
            PipelineError::ParseError(e) => write!(f, "{e}"),
        }
    }
}

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
impl From<WikipediaDumpReaderError> for PipelineError {
    fn from(value: WikipediaDumpReaderError) -> Self {
        Self::WikipediaDumpReaderError(value)
    }
}

#[derive(Debug)]
pub enum EmbeddingError {
    EmbeddingServiceError(EmbeddingServiceError),
}
impl StdError for EmbeddingError {}
impl Display for EmbeddingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            EmbeddingError::EmbeddingServiceError(e) => {
                write!(f, "{e}")
            }
        }
    }
}
impl From<EmbeddingError> for PipelineError {
    fn from(value: EmbeddingError) -> Self {
        Self::EmbeddingError(value)
    }
}

#[derive(Debug)]
pub enum LinkError {
    NoCurrentProgressBar,
}
impl StdError for LinkError {}
impl Display for LinkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            LinkError::NoCurrentProgressBar => {
                write!(f, "NoCurrentProgressBar")
            }
        }
    }
}
impl From<LinkError> for PipelineError {
    fn from(value: LinkError) -> Self {
        Self::LinkError(value)
    }
}
#[derive(Debug)]
pub enum CompressionError {
    Io(io::Error),
}
impl StdError for CompressionError {}
impl Display for CompressionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            CompressionError::Io(_e) => {
                write!(f, "NoCurrentProgressBar")
            }
        }
    }
}
impl From<CompressionError> for PipelineError {
    fn from(value: CompressionError) -> Self {
        Self::CompressionError(value)
    }
}

#[derive(Debug)]
pub enum Sql {
    Sql(sqlx::Error),
}
impl StdError for Sql {}
impl Display for Sql {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Sql::Sql(_e) => {
                write!(f, "NoCurrentProgressBar")
            }
        }
    }
}
impl From<Sql> for PipelineError {
    fn from(value: Sql) -> Self {
        Self::Sql(value)
    }
}

#[derive(Debug)]
pub enum ParseError {
    ParseError(String),
    Tokio(oneshot::error::RecvError),
    Timeout,
}
impl StdError for ParseError {}
impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ParseError::ParseError(e) => {
                write!(f, "Could Not Parse {e}")
            }
            ParseError::Timeout => write!(f, "Timeout"),
            ParseError::Tokio(e) => {
                write!(f, "{e}")
            }
        }
    }
}
impl From<ParseError> for PipelineError {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}
