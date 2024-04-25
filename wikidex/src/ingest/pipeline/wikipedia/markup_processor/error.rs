use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

#[derive(Debug)]
pub(crate) enum WikiMarkupProcessingError {}

// impl From<LlmClientError> for WikiMarkupProcessingError {
//     fn from(value: LlmClientError) -> Self {
//         Self::Llm(value)
//     }
// }
// impl From<EmbeddingServiceError> for WikiMarkupProcessingError {
//     fn from(value: EmbeddingServiceError) -> Self {
//         Self::Embed(value)
//     }
// }

impl Error for WikiMarkupProcessingError {}
impl Display for WikiMarkupProcessingError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> Result {
        Ok(())
    }
}
