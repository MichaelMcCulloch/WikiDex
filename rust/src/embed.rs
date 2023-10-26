use reqwest::{Client, Url};
use serde::Deserialize;
use std::{
    fmt::{self, Debug, Display, Formatter},
    path::Path,
    time::Duration,
};

#[derive(Deserialize)]
struct EmbeddingsResponse {
    pub(crate) embeddings: Vec<Vec<f32>>,
}
pub struct EmbedService {
    client: Client,
    host: Url,
}

impl EmbedService {
    pub fn new<S: AsRef<str>>(host: &S) -> Result<Self, url::ParseError> {
        let client = Client::new();
        let host = Url::parse(host.as_ref())?;
        Ok(Self { client, host })
    }
}

#[async_trait::async_trait]
pub trait Embed {
    type E;
    async fn embed(&self, str: &[&str]) -> Result<Vec<Vec<f32>>, Self::E>;
}

#[async_trait::async_trait]
impl Embed for EmbedService {
    type E = reqwest::Error;
    async fn embed(&self, query: &[&str]) -> Result<Vec<Vec<f32>>, reqwest::Error> {
        let payload = serde_json::json!({
            "sentences": query
        });
        let response: EmbeddingsResponse = self
            .client
            .post(self.host.clone())
            .timeout(Duration::from_secs(360))
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.embeddings)
    }
}
