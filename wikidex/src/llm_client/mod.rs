mod error;

#[cfg(feature = "openai")]
mod openai;
#[cfg(feature = "triton")]
mod triton;

#[cfg(feature = "openai")]
pub(crate) use openai::OpenAiInstructClient;
#[cfg(feature = "triton")]
pub(crate) use triton_client::Client as TritonClient;

use tokio::sync::mpsc::UnboundedSender;

use crate::openai::LanguageServiceArguments;
use error::LlmClientError;

pub(crate) trait LlmClientService {
    async fn get_response<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments<'_>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<String, LlmClientError>;

    async fn stream_response<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments<'_>,
        tx: UnboundedSender<String>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<(), LlmClientError>;
    fn fill_rag_template(&self, arguments: LanguageServiceArguments) -> String {
        let c1 = arguments.citation_index_begin + 1;
        let c2 = arguments.citation_index_begin + 2;
        let c3 = arguments.citation_index_begin + 3;
        let c4 = arguments.citation_index_begin + 4;

        arguments
            .system
            .replace("$$$USER_QUERY$$$", arguments.query)
            .replace("$$$URL$$$", "http://localhost")
            .replace("$$$CITE1$$$", &c1.to_string())
            .replace("$$$CITE2$$$", &c2.to_string())
            .replace("$$$CITE3$$$", &c3.to_string())
            .replace("$$$CITE4$$$", &c4.to_string())
            .replace("$$$DOCUMENT_LIST$$$", arguments.documents)
    }
}

pub(crate) struct LlmClient<Backend: LlmClientService> {
    client: Backend,
}

pub(crate) enum LlmClientKind {
    #[cfg(feature = "triton")]
    Triton(LlmClient<TritonClient>),
    #[cfg(feature = "openai")]
    OpenAiInstruct(LlmClient<OpenAiInstructClient>),
}

impl LlmClientService for LlmClientKind {
    async fn get_response<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments<'_>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<String, LlmClientError> {
        match self {
            #[cfg(feature = "triton")]
            LlmClientKind::Triton(t) => t.get_response(arguments, max_tokens, stop_phrases).await,
            #[cfg(feature = "openai")]
            LlmClientKind::OpenAiInstruct(o) => {
                o.get_response(arguments, max_tokens, stop_phrases).await
            }
        }
    }

    async fn stream_response<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments<'_>,
        tx: UnboundedSender<String>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<(), LlmClientError> {
        match self {
            #[cfg(feature = "triton")]
            LlmClientKind::Triton(t) => {
                t.stream_response(arguments, tx, max_tokens, stop_phrases)
                    .await
            }
            #[cfg(feature = "openai")]
            LlmClientKind::OpenAiInstruct(o) => {
                o.stream_response(arguments, tx, max_tokens, stop_phrases)
                    .await
            }
        }
    }
}
