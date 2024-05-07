use std::{sync::Arc, time::SystemTime};

use chrono::{DateTime, Utc};
use tera::{Context, Tera};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    RwLock,
};

use super::{
    LanguageServiceArguments, LanguageServiceDocument, LlmClientError, LlmMessage, LlmRole,
    OpenAiInstructClient, PartialLlmMessage, TritonClient,
};

pub trait LlmClientBackendKind {}
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
    pub(crate) async fn format_rag_template(
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
pub trait LlmClientService: LlmClientBackend {
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

        tokio::spawn(async move {
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

pub struct LlmClient<Backend: LlmClientBackendKind> {
    pub(crate) client: Backend,
    pub(crate) tera: Arc<RwLock<Tera>>,
    // prompt_template: tera::Template
}

pub enum LlmClientImpl {
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
