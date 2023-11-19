use std::{error::Error, time::Duration};

use async_openai::config::{Config, OpenAIConfig};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::llm::LlmMessage;

use super::SynchronousOpenAiClientError::{self, ReqwestError};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct LlmResponseUsage {
    prompt_tokens: usize,
    total_tokens: usize,
    completion_tokens: usize,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct LlmResponseChoices {
    pub(crate) index: usize,
    pub(crate) message: LlmMessage,
    pub(crate) finish_reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct LlmResponse {
    pub(crate) id: String,
    pub(crate) object: String,
    pub(crate) created: usize,
    pub(crate) model: String,
    pub(crate) choices: Vec<LlmResponseChoices>,
    pub(crate) usage: LlmResponseUsage,
}

pub(crate) struct SyncOpenAiClient {
    client: Client,
    config: OpenAIConfig,
}

impl SyncOpenAiClient {
    pub(crate) fn with_config(config: OpenAIConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(360))
                .build()
                .unwrap(),
            config,
        }
    }
}

pub(crate) trait OpenAIClient {
    type E: Error;
    fn test(&self, model: &str) -> Result<(), Self::E>;
    fn get_completion_for_conversation(
        &self,
        input: Vec<LlmMessage>,
        model: &str,
        max_tokens: u16,
    ) -> Result<LlmResponse, Self::E>;
}

impl OpenAIClient for SyncOpenAiClient {
    type E = SynchronousOpenAiClientError;
    fn test(&self, model: &str) -> Result<(), Self::E> {
        self.client
            .post(self.config.url("/chat/completions"))
            .json(&json!({
                "model": model,
                "messages": r#"[
                    { 
                      "role": "system",
                      "content": "faux HEAD request"
                    },
                    {
                      "role": "user",
                      "content": "Do not answer"
                    }
                ]"#,
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

    fn get_completion_for_conversation(
        &self,
        input: Vec<LlmMessage>,
        model: &str,
        max_tokens: u16,
    ) -> Result<LlmResponse, Self::E> {
        let response: LlmResponse = self
            .client
            .post(self.config.url("/chat/completions"))
            .json(&json!({
                "model": model,
                "messages": input,
                "max_tokens": max_tokens,
                "temperature": 1,
                "top_p": 1,
                "frequency_penalty": 0,
                "presence_penalty": 0
            }))
            .headers(self.config.headers())
            .send()
            .map_err(ReqwestError)?
            .json()
            .map_err(ReqwestError)?;

        Ok(response)
    }
}
