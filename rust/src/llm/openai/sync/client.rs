use std::error::Error;

use async_openai::config::{Config, OpenAIConfig};
use reqwest::blocking::Client;

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
    fn test(&self) -> Result<(), Self::E>;
}

impl OpenAIClient for SyncOpenAiClient {
    type E = OpenAiClientError;
    fn test(&self) -> Result<(), Self::E> {
        self.client
            .head(self.config.api_base())
            .headers(self.config.headers())
            .send()
            .map_err(ReqwestError)?
            .text()
            .map_err(ReqwestError)?;

        Ok(())
    }
}
