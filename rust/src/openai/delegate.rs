use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use super::{
    chat::ChatCompletionClient,
    completion::CompletionClient,
    embedding::EmbeddingClient,
    protocol::{LlmMessage, PartialLlmMessage},
    EmbeddingServiceError, LlmRole, LlmServiceError,
};

pub(crate) struct LanguageServiceServiceArguments<'arg> {
    pub(crate) system: &'arg str,
    pub(crate) documents: &'arg str,
    pub(crate) query: &'arg str,
    pub(crate) citation_index_begin: usize,
}
pub(super) enum ECompletionClient {
    Chat(ChatCompletionClient),
    Completion(CompletionClient),
}

impl ECompletionClient {
    pub(crate) async fn get_response(
        &self,
        arguments: LanguageServiceServiceArguments<'_>,
    ) -> Result<String, LlmServiceError> {
        match self {
            ECompletionClient::Chat(client) => client.get_response(arguments).await,
            ECompletionClient::Completion(client) => client.get_response(arguments).await,
        }
    }

    pub(crate) async fn stream_response(
        &self,
        arguments: LanguageServiceServiceArguments<'_>,
        tx: UnboundedSender<String>,
    ) -> Result<(), LlmServiceError> {
        match self {
            ECompletionClient::Chat(client) => client.stream_response(arguments, tx).await,
            ECompletionClient::Completion(client) => client.stream_response(arguments, tx).await,
        }
    }
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

    pub(crate) async fn embed(&self, query: &str) -> Result<Vec<f32>, EmbeddingServiceError> {
        self.embed_client.embed(query).await
    }

    pub(crate) async fn get_llm_answer(
        &self,
        arguments: LanguageServiceServiceArguments<'_>,
    ) -> Result<LlmMessage, LlmServiceError> {
        let message = self.llm_client.get_response(arguments).await?;
        Ok(LlmMessage {
            role: LlmRole::Assistant,
            content: message,
        })
    }
    pub(crate) async fn stream_llm_answer(
        &self,
        arguments: LanguageServiceServiceArguments<'_>,
        tx: UnboundedSender<PartialLlmMessage>,
    ) -> Result<(), LlmServiceError> {
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
