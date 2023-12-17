use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use super::{
    embedding::EmbeddingClient,
    protocol::{LlmMessage, PartialLlmMessage},
    service::{ECompletionClient, EmbedService, LlmService, TCompletionClient},
    EmbeddingServiceError, LlmRole, LlmServiceError,
};

pub(crate) struct LanguageServiceServiceArguments<'arg> {
    pub(crate) system: &'arg str,
    pub(crate) documents: &'arg str,
    pub(crate) query: &'arg str,
    pub(crate) citation_index_begin: usize,
}

pub(crate) struct OpenAiDelegate {
    llm_client: ECompletionClient,
    embed_client: EmbeddingClient,
}

impl OpenAiDelegate {
    pub(super) fn new(llm_client: ECompletionClient, embed_client: EmbeddingClient) -> Self {
        OpenAiDelegate {
            llm_client,
            embed_client,
        }
    }
}

#[async_trait::async_trait]
impl EmbedService for OpenAiDelegate {
    type E = EmbeddingServiceError;
    async fn embed(&self, query: &str) -> Result<Vec<f32>, Self::E> {
        self.embed_client.embed(query).await
    }
}

#[async_trait::async_trait]
impl LlmService for OpenAiDelegate {
    type E = LlmServiceError;
    async fn get_llm_answer(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
    ) -> Result<LlmMessage, Self::E> {
        let message = self.llm_client.get_response(arguments).await?;
        Ok(LlmMessage {
            role: LlmRole::Assistant,
            content: message,
        })
    }
    async fn stream_llm_answer(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
        tx: UnboundedSender<PartialLlmMessage>,
    ) -> Result<(), Self::E> {
        let (tx_s, mut rx_s) = unbounded_channel();

        actix_web::rt::spawn(async move {
            while let Some(content) = rx_s.recv().await {
                let _ = tx.send(PartialLlmMessage {
                    role: None,
                    content: Some(content),
                });
            }
        });
        self.llm_client.stream_response(arguments, tx_s).await
    }
}
