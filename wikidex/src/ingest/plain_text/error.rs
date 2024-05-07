use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

use crate::llm_client::LlmClientError;
use nebula_client::v3::graph::GraphQueryError;
use nebula_fbthrift_graph_v3::graph_service::AuthenticateError;

use crate::embedding_client::EmbeddingServiceError;

#[derive(Debug)]
pub(crate) enum PlainTextProcessingError {
    Llm(LlmClientError),
    Embed(EmbeddingServiceError),
    Io(std::io::Error),
    NebulaAuthentication(AuthenticateError),
    GraphQueryError(GraphQueryError),
    MalformedAddress,
}

impl From<LlmClientError> for PlainTextProcessingError {
    fn from(value: LlmClientError) -> Self {
        Self::Llm(value)
    }
}
impl From<EmbeddingServiceError> for PlainTextProcessingError {
    fn from(value: EmbeddingServiceError) -> Self {
        Self::Embed(value)
    }
}
impl From<std::io::Error> for PlainTextProcessingError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<AuthenticateError> for PlainTextProcessingError {
    fn from(value: AuthenticateError) -> Self {
        Self::NebulaAuthentication(value)
    }
}
impl From<GraphQueryError> for PlainTextProcessingError {
    fn from(value: GraphQueryError) -> Self {
        Self::GraphQueryError(value)
    }
}

impl Error for PlainTextProcessingError {}
impl Display for PlainTextProcessingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PlainTextProcessingError::Llm(e) => write!(f, "{:?}", e),
            PlainTextProcessingError::Embed(e) => write!(f, "{:?}", e),
            PlainTextProcessingError::Io(e) => write!(f, "{:?}", e),
            PlainTextProcessingError::NebulaAuthentication(e) => write!(f, "{:?}", e),
            PlainTextProcessingError::GraphQueryError(e) => write!(f, "{:?}", e),
            PlainTextProcessingError::MalformedAddress => todo!(),
        }
    }
}
