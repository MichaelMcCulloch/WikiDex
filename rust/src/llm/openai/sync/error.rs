use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum SynchronousOpenAiClientError {
    ReqwestError(reqwest::Error),
    EmptyResponse,
}

impl std::error::Error for SynchronousOpenAiClientError {}

impl Display for SynchronousOpenAiClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SynchronousOpenAiClientError::ReqwestError(err) => {
                write!(f, "SynchronousOpenAiClientError: {}", err)
            }
            SynchronousOpenAiClientError::EmptyResponse => {
                write!(
                    f,
                    "SynchronousOpenAiClientError: Empty Response from service"
                )
            }
        }
    }
}