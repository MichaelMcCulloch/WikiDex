use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug)]
pub(crate) enum LlmClientError {
    #[cfg(feature = "triton")]
    Utf8Error(std::str::Utf8Error),
    #[cfg(feature = "triton")]
    Anyhow(anyhow::Error),
    #[cfg(feature = "triton")]
    TonicError(tonic::transport::Error),
    #[cfg(feature = "triton")]
    TonicStatus(tonic::Status),
    #[cfg(feature = "openai")]
    OpenAiClient(async_openai::error::OpenAIError),
}

#[cfg(feature = "triton")]
impl From<tonic::Status> for LlmClientError {
    fn from(value: tonic::Status) -> Self {
        Self::TonicStatus(value)
    }
}

#[cfg(feature = "triton")]
impl From<std::str::Utf8Error> for LlmClientError {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}
#[cfg(feature = "triton")]
impl From<anyhow::Error> for LlmClientError {
    fn from(value: anyhow::Error) -> Self {
        Self::Anyhow(value)
    }
}
#[cfg(feature = "triton")]
impl From<tonic::transport::Error> for LlmClientError {
    fn from(value: tonic::transport::Error) -> Self {
        Self::TonicError(value)
    }
}
#[cfg(feature = "openai")]
impl From<async_openai::error::OpenAIError> for LlmClientError {
    fn from(value: async_openai::error::OpenAIError) -> Self {
        Self::OpenAiClient(value)
    }
}

impl std::error::Error for LlmClientError {}

impl Display for LlmClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "triton")]
            LlmClientError::Utf8Error(e) => write!(f, "LlmClientError: Utf8Error: {e:?}"),
            #[cfg(feature = "triton")]
            LlmClientError::Anyhow(e) => write!(f, "LlmClientError: Anyhow: {e:?}"),
            #[cfg(feature = "triton")]
            LlmClientError::TonicError(e) => write!(f, "LlmClientError: TonicError: {e:?}"),
            #[cfg(feature = "triton")]
            LlmClientError::TonicStatus(e) => write!(f, "LlmClientError: TonicStatus: {e:?}"),
            #[cfg(feature = "openai")]
            LlmClientError::OpenAiClient(e) => write!(f, "LlmClientError: OpenAiClient: {e}"),
        }
    }
}
