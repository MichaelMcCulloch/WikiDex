use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
    io,
    path::PathBuf,
};

#[derive(Debug)]
pub(crate) enum WikiMarkupProcessingError {
    XmlNotFound(PathBuf),
}

impl Error for WikiMarkupProcessingError {}
impl Display for WikiMarkupProcessingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            WikiMarkupProcessingError::XmlNotFound(path) => {
                write!(f, "IngestEngine: Input XML '{}' not found", path.display())
            }
        }
    }
}
