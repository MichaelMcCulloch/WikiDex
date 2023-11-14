use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
    path::PathBuf,
};

#[derive(Debug)]
pub(crate) enum IngestError {
    XMLNotFound(PathBuf),
    OutputDirectoryNotFound(PathBuf),
}

impl Error for IngestError {}
impl Display for IngestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            IngestError::XMLNotFound(path) => {
                write!(f, "IngestEngine: Input XML '{}' not found", path.display())
            }
            IngestError::OutputDirectoryNotFound(path) => {
                write!(
                    f,
                    "IngestEngine: Output directory '{}' not found",
                    path.display()
                )
            }
        }
    }
}
