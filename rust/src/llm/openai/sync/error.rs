use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum OpenAiClientError {
    ReqwestError(reqwest::Error),
    EmptyResponse,
}

impl std::error::Error for OpenAiClientError {}

impl Display for OpenAiClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            OpenAiClientError::ReqwestError(err) => write!(f, "OpenAiClient: {}", err),
            OpenAiClientError::EmptyResponse => {
                write!(f, "OpenAiClient: Empty Response from service")
            }
        }
    }
}
