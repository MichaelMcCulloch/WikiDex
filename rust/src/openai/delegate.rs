use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use super::{
    chat::ChatClient,
    embedding::EmbeddingClient,
    instruct::InstructClient,
    protocol::{LlmMessage, PartialLlmMessage},
    EmbeddingServiceError, LlmRole, LlmServiceError,
};

pub(crate) struct LanguageServiceArguments<'arg> {
    pub(crate) system: &'arg str,
    pub(crate) documents: &'arg str,
    pub(crate) query: &'arg str,
    pub(crate) citation_index_begin: usize,
}
pub(super) enum LlmClient {
    Chat(ChatClient),
    Instruct(InstructClient),
}

impl LlmClient {
    pub(crate) async fn get_response(
        &self,
        arguments: LanguageServiceArguments<'_>,
    ) -> Result<String, LlmServiceError> {
        match self {
            LlmClient::Chat(chat) => chat.get_response(arguments).await,
            LlmClient::Instruct(instruct) => instruct.get_response(arguments).await,
        }
    }

    pub(crate) async fn stream_response(
        &self,
        arguments: LanguageServiceArguments<'_>,
        tx: UnboundedSender<String>,
    ) -> Result<(), LlmServiceError> {
        match self {
            LlmClient::Chat(chat) => chat.stream_response(arguments, tx).await,
            LlmClient::Instruct(instruct) => instruct.stream_response(arguments, tx).await,
        }
    }
}

pub(crate) struct OpenAiDelegate {
    llm_client: LlmClient,
    embed_client: EmbeddingClient,
}

impl OpenAiDelegate {
    pub(super) fn new(llm_client: LlmClient, embed_client: EmbeddingClient) -> Self {
        OpenAiDelegate {
            llm_client,
            embed_client,
        }
    }

    pub(crate) async fn embed(&self, query: &str) -> Result<Vec<f32>, EmbeddingServiceError> {
        self.embed_client.embed(query).await
    }
    pub(crate) async fn embed_batch(
        &self,
        queries: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, EmbeddingServiceError> {
        self.embed_client.embed_batch(queries).await
    }

    pub(crate) async fn get_llm_answer(
        &self,
        arguments: LanguageServiceArguments<'_>,
    ) -> Result<LlmMessage, LlmServiceError> {
        let message = self.llm_client.get_response(arguments).await?;
        Ok(LlmMessage {
            role: LlmRole::Assistant,
            content: message,
        })
    }
    pub(crate) async fn stream_llm_answer(
        &self,
        arguments: LanguageServiceArguments<'_>,
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
