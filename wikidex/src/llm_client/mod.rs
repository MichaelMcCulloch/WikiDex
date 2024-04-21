mod arguments;
mod error;
mod kind;
mod protocol;

#[cfg(feature = "openai")]
mod openai;
#[cfg(feature = "triton")]
mod triton;
#[cfg(feature = "triton")]
mod triton_helper;

#[cfg(feature = "openai")]
pub(crate) use openai::OpenAiInstructClient;
#[cfg(feature = "triton")]
use tonic::transport::Channel;
#[cfg(feature = "triton")]
pub(crate) use trtllm::triton::grpc_inference_service_client::GrpcInferenceServiceClient;

use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

pub(crate) use arguments::LanguageServiceArguments;
pub(crate) use error::LlmClientError;
pub(crate) use kind::ModelKind;
pub(crate) use protocol::{LlmMessage, LlmRole, PartialLlmMessage};

#[cfg(feature = "triton")]
pub(crate) type TritonClient = GrpcInferenceServiceClient<Channel>;

pub(crate) trait LlmClientBackendKind {}
pub(crate) trait LlmClientBackend {
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
}

impl<T> LlmClientService for T where T: LlmClientBackend {}
pub(crate) trait LlmClientService: LlmClientBackend {
    async fn get_llm_answer<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments<'_>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<LlmMessage, LlmClientError> {
        let message = self
            .get_response(arguments, max_tokens, stop_phrases)
            .await?;
        Ok(LlmMessage {
            role: LlmRole::Assistant,
            content: message,
        })
    }
    async fn stream_llm_answer<S: AsRef<str>>(
        &self,
        arguments: LanguageServiceArguments<'_>,
        tx: UnboundedSender<PartialLlmMessage>,
        max_tokens: u16,
        stop_phrases: Vec<S>,
    ) -> Result<(), LlmClientError> {
        let (tx_s, mut rx_s) = unbounded_channel();

        actix_web::rt::spawn(async move {
            while let Some(content) = rx_s.recv().await {
                let _ = tx.send(PartialLlmMessage {
                    role: None,
                    content: Some(content),
                });
            }
        });
        self.stream_response(arguments, tx_s, max_tokens, stop_phrases)
            .await
    }

    fn fill_rag_template(&self, arguments: LanguageServiceArguments) -> String {
        let mut replace_query = arguments
            .system
            .replace("$$$USER_QUERY$$$", arguments.query);

        for (index, source) in arguments.indices.iter().enumerate() {
            replace_query = replace_query.replace(
                format!("$$$CITE{}$$$", index + 1).as_str(),
                format!("{}", source).as_str(),
            );
        }

        replace_query.replace("$$$DOCUMENT_LIST$$$", arguments.documents)
    }
}

pub(crate) struct LlmClient<Backend: LlmClientBackendKind> {
    client: Backend,
}

pub(crate) enum LlmClientKind {
    #[cfg(feature = "triton")]
    Triton(LlmClient<TritonClient>),
    #[cfg(feature = "openai")]
    OpenAiInstruct(LlmClient<OpenAiInstructClient>),
}
impl LlmClientBackend for LlmClientKind {
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
