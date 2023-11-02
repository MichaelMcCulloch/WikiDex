use std::{fmt::Display, path::PathBuf};

use colored::Colorize;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Config {
    pub(crate) engine: EngineConfig,
    pub(crate) embed: EmbedConfig,
    pub(crate) llm: LlmConfig,
    pub(crate) ui: UiConfig,
}
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct EngineConfig {
    pub(crate) protocol: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) index: PathBuf,
    pub(crate) docstore: PathBuf,
    pub(crate) conversation_path: String,
    pub(crate) query_path: String,
}
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct EmbedConfig {
    pub(crate) protocol: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) model: String,
    pub(crate) batch_size: u16,
    pub(crate) path: String,
}
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct LlmConfig {
    pub(crate) protocol: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) path: String,
    pub(crate) model: String,
}
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct UiConfig {
    pub(crate) protocol: String,
    pub(crate) host: String,
    pub(crate) port: u16,
}

pub(crate) trait ConfigUrl {
    fn url(&self) -> Url;
}

impl ConfigUrl for EngineConfig {
    fn url(&self) -> Url {
        let EngineConfig {
            host,
            port,
            protocol,
            ..
        } = self;

        Url::parse(&format!("{protocol}://{host}:{port}/")).unwrap()
    }
}

impl ConfigUrl for LlmConfig {
    fn url(&self) -> Url {
        let LlmConfig {
            host,
            port,
            protocol,
            ..
        } = self;

        Url::parse(&format!("{protocol}://{host}:{port}/")).unwrap()
    }
}
impl ConfigUrl for EmbedConfig {
    fn url(&self) -> Url {
        let EmbedConfig {
            host,
            port,
            protocol,
            ..
        } = self;

        Url::parse(&format!("{protocol}://{host}:{port}/")).unwrap()
    }
}

impl ConfigUrl for UiConfig {
    fn url(&self) -> Url {
        let UiConfig {
            host,
            port,
            protocol,
            ..
        } = self;

        Url::parse(&format!("{protocol}://{host}:{port}/")).unwrap()
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Config {
            engine:
                EngineConfig {
                    index: engine_index,
                    docstore: engine_docstore,
                    conversation_path: engine_conversation_path,
                    query_path: engine_query_path,
                    ..
                },
            embed:
                EmbedConfig {
                    model: embed_model,
                    batch_size: embed_batch_size,
                    ..
                },
            llm: LlmConfig {
                model: llm_model, ..
            },
            ..
        } = self;

        let engine_index = engine_index.display();
        let engine_docstore = engine_docstore.display();

        let engine_url: Url = self.engine.url();
        let engine_url = engine_url.as_str().yellow();
        let embed_url: Url = self.embed.url();
        let embed_url = embed_url.as_str().yellow();
        let llm_url: Url = self.llm.url();
        let llm_url = llm_url.as_str().yellow();
        let ui_url: Url = self.ui.url();
        let ui_url = ui_url.as_str().yellow();

        write!(
            f,
            "Engine running at {engine_url}.\n\tServing conversations on /{engine_conversation_path}.\n\tService queries on /{engine_query_path}.\n\tUsing index at {engine_index}.\n\tUsing docstore at {engine_docstore}.\nUsing embedding service at {embed_url}.\n\tUsing embedder {embed_model} with a batch size of {embed_batch_size}.\nUsing llm service at {llm_url}.\n\tUsing {llm_model}.\nUi running at {ui_url}.",
        )
    }
}
