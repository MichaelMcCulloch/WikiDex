use reqwest::{blocking::Client, Url};
use serde::Deserialize;
use std::time::Duration;

use super::{service::EmbedServiceSync, EmbeddingServiceError};

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
}

#[derive(Deserialize)]
struct EmbeddingsResponse {
    pub(crate) embeddings: Vec<Vec<f32>>,
}

impl EmbedServiceSync for Embedder {
    type E = EmbeddingServiceError;
    fn embed_batch(&self, query: &[&str]) -> Result<Vec<Vec<f32>>, Self::E> {
        let payload = serde_json::json!({
            "sentences": query
        });

        let response: EmbeddingsResponse = self
            .client
            .post(self.host.clone())
            .timeout(Duration::from_secs(180))
            .json(&payload)
            .send()
            .map_err(|e| EmbeddingServiceError::Reqwuest(e))?
            .json()
            .map_err(|e| EmbeddingServiceError::Reqwuest(e))?;

        if response.embeddings.len() != query.len() {
            Err(EmbeddingServiceError::EmbeddingSizeMismatch(
                query.len(),
                response.embeddings.len(),
            ))
        } else {
            Ok(response.embeddings)
        }
    }
}
