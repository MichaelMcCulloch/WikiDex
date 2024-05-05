mod arguments;
mod endpoint;
mod error;
mod kind;
mod openai;
mod protocol;
mod triton;
mod triton_helper;

use std::{sync::Arc, time::SystemTime};

use chrono::{DateTime, Utc};
pub(crate) use endpoint::ModelEndpoint;
pub(crate) use openai::OpenAiInstructClient;

use tera::{Context, Tera};
use tonic::transport::Channel;

pub(crate) use trtllm::triton::grpc_inference_service_client::GrpcInferenceServiceClient;

use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    RwLock,
};

pub(crate) use arguments::{LanguageServiceArguments, LanguageServiceDocument};
pub(crate) use error::LlmClientError;
pub(crate) use kind::ModelKind;
pub(crate) use protocol::{LlmMessage, LlmRole, PartialLlmMessage};

pub(crate) type TritonClient = GrpcInferenceServiceClient<Channel>;

pub(crate) trait LlmClientBackendKind {}
pub(crate) trait LlmClientBackend {
    async fn get_response(
        &self,
        arguments: LanguageServiceArguments,
    ) -> Result<String, LlmClientError>;

    async fn stream_response(
        &self,
        arguments: LanguageServiceArguments,
        tx: UnboundedSender<String>,
    ) -> Result<(), LlmClientError>;
}

impl LlmClient<TritonClient> {
    async fn format_rag_template(
        &self,
        messages: &Vec<LlmMessage>,
        documents: &Vec<LanguageServiceDocument>,
        user_query: &String,
    ) -> Result<String, LlmClientError> {
        let mut system_context = Context::new();
        system_context.insert("documents", documents);
        system_context.insert("user_query", user_query);
        system_context.insert(
            "current_time",
            &DateTime::<Utc>::from(SystemTime::now()).to_rfc3339(),
        );
        let system_message = self
            .tera
            .read()
            .await
            .render("markdown.md.j2", &system_context)?;

        let mut prompt_context = Context::new();
        prompt_context.insert("system_message", &system_message);
        prompt_context.insert("messages", &messages);
        prompt_context.insert("bos_token", "<s>");
        prompt_context.insert("eos_token", "</s>");

        let prompt = self.tera.read().await.render("chat.j2", &prompt_context)?;
        log::info!("{prompt}");
        Ok(prompt)
    }
}

impl<T> LlmClientService for T where T: LlmClientBackend {}
pub(crate) trait LlmClientService: LlmClientBackend {
    async fn get_llm_answer(
        &self,
        arguments: LanguageServiceArguments,
    ) -> Result<LlmMessage, LlmClientError> {
        let message = self.get_response(arguments).await?;
        Ok(LlmMessage {
            role: LlmRole::Assistant,
            content: message,
        })
    }
    async fn stream_llm_answer(
        &self,
        arguments: LanguageServiceArguments,
        tx: UnboundedSender<PartialLlmMessage>,
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
        self.stream_response(arguments, tx_s).await
    }
}

pub(crate) struct LlmClient<Backend: LlmClientBackendKind> {
    client: Backend,
    tera: Arc<RwLock<Tera>>,
    // prompt_template: tera::Template
}

pub(crate) enum LlmClientImpl {
    Triton(LlmClient<TritonClient>),

    OpenAiInstruct(LlmClient<OpenAiInstructClient>),
}
impl LlmClientBackend for LlmClientImpl {
    async fn get_response(
        &self,
        arguments: LanguageServiceArguments,
    ) -> Result<String, LlmClientError> {
        match self {
            LlmClientImpl::Triton(t) => t.get_response(arguments).await,

            LlmClientImpl::OpenAiInstruct(o) => o.get_response(arguments).await,
        }
    }

    async fn stream_response(
        &self,
        arguments: LanguageServiceArguments,
        tx: UnboundedSender<String>,
    ) -> Result<(), LlmClientError> {
        match self {
            LlmClientImpl::Triton(t) => t.stream_response(arguments, tx).await,

            LlmClientImpl::OpenAiInstruct(o) => o.stream_response(arguments, tx).await,
        }
    }
}
