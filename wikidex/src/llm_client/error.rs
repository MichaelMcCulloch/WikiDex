use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug)]
pub enum LlmClientError {
    Utf8Error(std::str::Utf8Error),
    Anyhow(anyhow::Error),
    TonicError(tonic::transport::Error),
    TonicStatus(tonic::Status),
    OpenAiClient(async_openai::error::OpenAIError),
    Tera(tera::Error),
    EmptyResponse,
}

impl From<tonic::Status> for LlmClientError {
    fn from(value: tonic::Status) -> Self {
        Self::TonicStatus(value)
    }
}

impl From<std::str::Utf8Error> for LlmClientError {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}

impl From<anyhow::Error> for LlmClientError {
    fn from(value: anyhow::Error) -> Self {
        Self::Anyhow(value)
    }
}

impl From<tonic::transport::Error> for LlmClientError {
    fn from(value: tonic::transport::Error) -> Self {
        Self::TonicError(value)
    }
}

impl From<async_openai::error::OpenAIError> for LlmClientError {
    fn from(value: async_openai::error::OpenAIError) -> Self {
        Self::OpenAiClient(value)
    }
}
impl From<tera::Error> for LlmClientError {
    fn from(value: tera::Error) -> Self {
        Self::Tera(value)
    }
}

impl std::error::Error for LlmClientError {}

impl Display for LlmClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LlmClientError::Utf8Error(e) => write!(f, "LlmClientError: Utf8Error: {e:?}"),
            LlmClientError::Anyhow(e) => write!(f, "LlmClientError: Anyhow: {e:?}"),
            LlmClientError::TonicError(e) => write!(f, "LlmClientError: TonicError: {e:?}"),
            LlmClientError::TonicStatus(e) => write!(f, "LlmClientError: TonicStatus: {e:?}"),
            LlmClientError::OpenAiClient(e) => write!(f, "LlmClientError: OpenAiClient: {e}"),
            LlmClientError::EmptyResponse => write!(f, "LlmClientError: Empty Response"),
            LlmClientError::Tera(e) => write!(f, "LlmClientError: Tera: {e:?}"),
        }
    }
}
