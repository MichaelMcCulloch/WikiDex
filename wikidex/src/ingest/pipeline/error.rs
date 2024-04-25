use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result},
    io,
};

use crate::embedding_client::EmbeddingServiceError;

use super::wikipedia::WikiMarkupProcessingError;

#[derive(Debug)]
pub enum PipelineError {
    LinkError(LinkError),
    EmbeddingError(EmbeddingError),
    CompressionError(CompressionError),
    WikipediaDumpReaderError(WikipediaDumpReaderError),
    WikipediaMarkupParseError(WikipediaMarkupParseError),
    WikipediaHeadingSplitterError(WikipediaHeadingSplitterError),
    Sql(Sql),
}
impl StdError for PipelineError {}
impl Display for PipelineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PipelineError::WikipediaDumpReaderError(e) => {
                write!(f, "{e}")
            }
            PipelineError::Sql(e) => write!(f, "{e}"),
            PipelineError::LinkError(e) => write!(f, "{e}"),
            PipelineError::EmbeddingError(e) => write!(f, "{e}"),
            PipelineError::CompressionError(e) => write!(f, "{e}"),
            PipelineError::WikipediaMarkupParseError(e) => write!(f, "{e}"),
            PipelineError::WikipediaHeadingSplitterError(e) => write!(f, "{e}"),
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
                write!(f, "EmbeddingError::EmbeddingServiceError {e}")
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
                write!(f, "LinkError::NoCurrentProgressBar")
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
            CompressionError::Io(e) => {
                write!(f, "CompressionError::Io {e}")
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
pub enum WikipediaMarkupParseError {
    ParseError(String),
    NoContent(String, String),
    Redirect(String),
    None,
    Timeout(String),
}
impl StdError for WikipediaMarkupParseError {}
impl Display for WikipediaMarkupParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            WikipediaMarkupParseError::ParseError(e) => {
                write!(f, "ParseError {e}")
            }
            WikipediaMarkupParseError::Timeout(e) => write!(f, "Timeout: {e}"),
            WikipediaMarkupParseError::Redirect(e) => write!(f, "Redirect: {e}"),
            WikipediaMarkupParseError::NoContent(e, t) => write!(f, "No Content: {e}\n\n{t}."),
            WikipediaMarkupParseError::None => {
                write!(f, "Parser: Channel Closed")
            }
        }
    }
}
impl From<WikipediaMarkupParseError> for PipelineError {
    fn from(value: WikipediaMarkupParseError) -> Self {
        Self::WikipediaMarkupParseError(value)
    }
}

#[derive(Debug)]
pub enum WikipediaHeadingSplitterError {
    HeadingMismatch(String),
}
impl StdError for WikipediaHeadingSplitterError {}
impl Display for WikipediaHeadingSplitterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            WikipediaHeadingSplitterError::HeadingMismatch(e) => write!(f, "Heading Mismatch: {e}"),
        }
    }
}
impl From<WikipediaHeadingSplitterError> for PipelineError {
    fn from(value: WikipediaHeadingSplitterError) -> Self {
        Self::WikipediaHeadingSplitterError(value)
    }
}
