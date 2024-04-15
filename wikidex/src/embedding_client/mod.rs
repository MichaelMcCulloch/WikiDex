mod client;
mod error;

use async_openai::{error::OpenAIError, types::ListModelResponse};
pub(crate) use client::EmbeddingClient;
pub(crate) use error::EmbeddingServiceError;

pub(crate) trait EmbeddingClientService {
    async fn up(&self) -> Result<ListModelResponse, OpenAIError>;

    async fn embed_batch(
        &self,
        queries: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, EmbeddingServiceError>;
    async fn embed(&self, query: &str) -> Result<Vec<f32>, EmbeddingServiceError>;
}
