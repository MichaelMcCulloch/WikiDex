use reqwest::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum EmbeddingServiceError {
    Reqwuest(Error),
    EmbeddingSizeMismatch(usize, usize),
}

impl std::error::Error for EmbeddingServiceError {}

impl Display for EmbeddingServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            EmbeddingServiceError::Reqwuest(err) => {
                write!(f, "EmbeddingService: {}", err)
            }
            EmbeddingServiceError::EmbeddingSizeMismatch(expected, actual) => write!(
                f,
                "EmbeddingService: Embedding size mismatch. Expected: {}, Actual: {}",
                expected, actual
            ),
        }
    }
}
