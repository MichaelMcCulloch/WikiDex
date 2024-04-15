use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug)]
pub enum LlmClientError {
    #[cfg(feature = "triton")]
    TritonClient(triton_client::client::Error),
    #[cfg(feature = "openai")]
    OpenAiClient(async_openai::error::OpenAIError),
    EmptyResponse,
}

impl std::error::Error for LlmClientError {}

impl Display for LlmClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "triton")]
            LlmClientError::TritonClient(e) => write!(f, "LlmClientError: TritonClient: {e}"),
            #[cfg(feature = "openai")]
            LlmClientError::OpenAiClient(e) => write!(f, "LlmClientError: OpenAiClient: {e}"),
            LlmClientError::EmptyResponse => {
                write!(f, "LlmClientError: Empty Response from service")
            }
        }
    }
}
