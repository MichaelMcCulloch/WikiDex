use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
    io,
    path::PathBuf,
};

use crate::llm::LlmServiceError;

#[derive(Debug)]
pub(crate) enum WikiMarkupProcessingError {
    LlmError(LlmServiceError),
}

impl Error for WikiMarkupProcessingError {}
impl Display for WikiMarkupProcessingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            WikiMarkupProcessingError::LlmError(e) => {
                write!(f, "{e}")
            }
        }
    }
}
