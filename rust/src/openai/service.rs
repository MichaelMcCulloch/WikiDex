use std::error::Error;

use tokio::sync::mpsc::UnboundedSender;

use super::{
    chat::ChatCompletionClient,
    completion::CompletionClient,
    delegate::LanguageServiceServiceArguments,
    error::LlmServiceError,
    protocol::{LlmMessage, PartialLlmMessage},
};

#[async_trait::async_trait]
pub(crate) trait ChatService {
    type E: Error;
    async fn answer(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
    ) -> Result<LlmMessage, Self::E>;
    async fn stream_answer(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
        tx: UnboundedSender<PartialLlmMessage>,
    ) -> Result<(), Self::E>;
}

#[async_trait::async_trait]
pub(crate) trait CompletionService {
    type E: Error;
    async fn answer(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
    ) -> Result<LlmMessage, Self::E>;
    async fn stream_answer(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
        tx: UnboundedSender<PartialLlmMessage>,
    ) -> Result<(), Self::E>;
}

#[async_trait::async_trait]
pub(crate) trait EmbedService {
    type E: Error;
    async fn embed(&self, str: &str) -> Result<Vec<f32>, Self::E>;
}

#[async_trait::async_trait]
pub(crate) trait LlmService {
    type E: Error;
    async fn get_llm_answer(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
    ) -> Result<LlmMessage, Self::E>;
    async fn stream_llm_answer(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
        tx: UnboundedSender<PartialLlmMessage>,
    ) -> Result<(), Self::E>;
}

pub(super) enum ECompletionClient {
    Chat(ChatCompletionClient),
    Completion(CompletionClient),
}
#[async_trait::async_trait]
pub(super) trait TCompletionClient {
    async fn get_response(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
    ) -> Result<String, LlmServiceError>;
    async fn stream_response(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
        tx: UnboundedSender<String>,
    ) -> Result<(), LlmServiceError>;
}

#[async_trait::async_trait]
impl TCompletionClient for ECompletionClient {
    async fn get_response(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
    ) -> Result<String, LlmServiceError> {
        match self {
            ECompletionClient::Chat(client) => client.get_response(arguments).await,
            ECompletionClient::Completion(client) => client.get_response(arguments).await,
        }
    }

    async fn stream_response(
        &self,
        arguments: LanguageServiceServiceArguments<'async_trait>,
        tx: UnboundedSender<String>,
    ) -> Result<(), LlmServiceError> {
        match self {
            ECompletionClient::Chat(client) => client.stream_response(arguments, tx).await,
            ECompletionClient::Completion(client) => client.stream_response(arguments, tx).await,
        }
    }
}
