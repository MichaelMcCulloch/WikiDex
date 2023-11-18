use std::error::Error;

use async_openai::config::{Config, OpenAIConfig};
use reqwest::blocking::Client;
use serde_json::json;

use super::OpenAiClientError::{self, ReqwestError};

pub(crate) struct SyncOpenAiClient {
    client: Client,
    config: OpenAIConfig,
}

impl SyncOpenAiClient {
    pub(crate) fn with_config(config: OpenAIConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
}

pub(crate) trait OpenAIClient {
    type E: Error;
    fn test(&self, model: &str) -> Result<(), Self::E>;
}

impl OpenAIClient for SyncOpenAiClient {
    type E = OpenAiClientError;
    fn test(&self, model: &str) -> Result<(), Self::E> {
        self.client
            .post(self.config.url("/completions"))
            .json(&json!({
                "model": model,
                "prompt": "Faux Head Request.",
                "max_tokens": 1,
                "temperature": 0
            }))
            .headers(self.config.headers())
            .send()
            .map_err(ReqwestError)?
            .bytes()
            .map_err(ReqwestError)?;

        Ok(())
    }
}
