use reqwest::{Client, Url};
use serde::Deserialize;
use std::{
    fmt::{self, Display, Formatter},
    time::Duration,
};

#[derive(Deserialize)]
struct EmbeddingsResponse {
    pub(crate) embeddings: Vec<Vec<f32>>,
}
pub struct Embedder {
    client: Client,
    host: Url,
}

impl Embedder {
    pub(crate) fn new(host: Url) -> Result<Self, url::ParseError> {
        let start = std::time::Instant::now();

        let client = Client::new();

        let embedder = Self { client, host };

        log::info!("Connect Embedder {:?}", start.elapsed());
        Ok(embedder)
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
            .timeout(Duration::from_secs(180))
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
            EmbeddingServiceError::Reqwuest(err) => {
                write!(f, "EmbeddingService: {}", err)
            }
            EmbeddingServiceError::EmbeddingSizeMismatch(expected, actual) => write!(
                f,
                "EmbeddingService: Embedding size mismatch. Expected: {}, Actual: {}",
                expected, actual
            ),
        }
    }
}
