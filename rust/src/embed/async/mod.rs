use reqwest::{Client, Url};
use serde::Deserialize;
use std::time::Duration;

pub(crate) mod openai;

use super::{EmbedService, EmbeddingServiceError};

pub struct Embedder {
    client: Client,
    host: Url,
}

impl Embedder {
    pub(crate) fn new(host: Url) -> Result<Self, url::ParseError> {
        let client = Client::new();
        let embedder = Self { client, host };
        Ok(embedder)
    }

    async fn call_embedder(
        &self,
        queries: &[&str],
    ) -> Result<EmbeddingsResponse, <Self as EmbedService>::E> {
        let payload = serde_json::json!({
            "sentences": queries
        });
        let response: EmbeddingsResponse = self
            .client
            .post(self.host.clone())
            .timeout(Duration::from_secs(180))
            .json(&payload)
            .send()
            .await
            .map_err(|e| EmbeddingServiceError::Reqwuest(e))?
            .json()
            .await
            .map_err(|e| EmbeddingServiceError::Reqwuest(e))?;
        Ok(response)
    }
}

#[derive(Deserialize)]
struct EmbeddingsResponse {
    pub(crate) embeddings: Vec<Vec<f32>>,
}

#[async_trait::async_trait]
impl EmbedService for Embedder {
    type E = EmbeddingServiceError;
    async fn embed_batch(&self, queries: &[&str]) -> Result<Vec<Vec<f32>>, Self::E> {
        let response = self.call_embedder(queries).await?;

        if response.embeddings.len() != queries.len() {
            Err(EmbeddingServiceError::EmbeddingSizeMismatch(
                queries.len(),
                response.embeddings.len(),
            ))
        } else {
            Ok(response.embeddings)
        }
    }
    async fn embed(&self, query: &str) -> Result<Vec<f32>, Self::E> {
        let response = self.call_embedder(&[query]).await?;

        if response.embeddings.len() > 1 {
            Err(EmbeddingServiceError::EmbeddingSizeMismatch(
                1,
                response.embeddings.len(),
            ))
        } else if let Some(embedding) = response.embeddings.into_iter().next() {
            Ok(embedding)
        } else {
            Err(EmbeddingServiceError::EmbeddingSizeMismatch(1, 0))
        }
    }
}
