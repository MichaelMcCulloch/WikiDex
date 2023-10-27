use reqwest::{Client, Url};
use serde::Deserialize;
use std::{
    fmt::{self, Display, Formatter},
    time::Duration,
};

use crate::config::EmbedConfig;

#[derive(Deserialize)]
struct EmbeddingsResponse {
    pub(crate) embeddings: Vec<Vec<f32>>,
}
pub struct Embedder {
    client: Client,
    host: Url,
}

impl Embedder {
    pub(crate) fn new(config: EmbedConfig) -> Result<Self, url::ParseError> {
        let host: Url = config.into();
        let client = Client::new();

        Ok(Self { client, host })
    }
}

#[async_trait::async_trait]
pub(crate) trait EmbedService {
    type E;
    async fn embed(&self, str: &[&str]) -> Result<Vec<Vec<f32>>, Self::E>;
}

#[async_trait::async_trait]
impl EmbedService for Embedder {
    type E = EmbeddingServiceError;
    async fn embed(&self, query: &[&str]) -> Result<Vec<Vec<f32>>, Self::E> {
        let payload = serde_json::json!({
            "sentences": query
        });
        let response: EmbeddingsResponse = self
            .client
            .post(self.host.clone())
            .timeout(Duration::from_secs(360))
            .json(&payload)
            .send()
            .await
            .map_err(|e| EmbeddingServiceError::Reqwuest(e))?
            .json()
            .await
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

#[derive(Debug)]
pub(crate) enum EmbeddingServiceError {
    Reqwuest(reqwest::Error),
    EmbeddingSizeMismatch(usize, usize),
}

impl std::error::Error for EmbeddingServiceError {}

impl Display for EmbeddingServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EmbeddingServiceError::Reqwuest(reqwest_error) => write!(f, "{:?}", reqwest_error),
            EmbeddingServiceError::EmbeddingSizeMismatch(expected, received) => write!(
                f,
                "Embedding count does not match query count, expected {expected}, received {received}."
            ),
        }
    }
}
