use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug)]
pub(crate) enum LlmClientError {
    #[cfg(feature = "triton")]
    Utf8Error(std::str::Utf8Error),
    #[cfg(feature = "triton")]
    Anyhow(anyhow::Error),
    #[cfg(feature = "triton")]
    TritonClient(trtllm::error::AppError),
    #[cfg(feature = "triton")]
    TonicError(tonic::transport::Error),
    #[cfg(feature = "triton")]
    TonicStatus(tonic::Status),
    #[cfg(feature = "openai")]
    OpenAiClient(async_openai::error::OpenAIError),
    EmptyResponse,
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
            LlmClientError::TritonClient(e) => write!(f, "LlmClientError: TritonClient: {e:?}"),
            #[cfg(feature = "triton")]
            LlmClientError::TonicError(e) => write!(f, "LlmClientError: TonicError: {e:?}"),
            #[cfg(feature = "triton")]
            LlmClientError::TonicStatus(e) => write!(f, "LlmClientError: TonicStatus: {e:?}"),
            #[cfg(feature = "openai")]
            LlmClientError::OpenAiClient(e) => write!(f, "LlmClientError: OpenAiClient: {e}"),
            LlmClientError::EmptyResponse => {
                write!(f, "LlmClientError: Empty Response from service")
            }
        }
    }
}
