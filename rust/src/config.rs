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

impl Into<Url> for EngineConfig {
    fn into(self) -> Url {
        let EngineConfig {
            protocol,
            host,
            port,
            conversation_path,
            ..
        } = self;

        Url::parse(&format!("{protocol}://{host}:{port}/{conversation_path}")).unwrap()
    }
}

impl Into<Url> for LlmConfig {
    fn into(self) -> Url {
        let LlmConfig {
            host,
            port,
            path,
            protocol,
            ..
        } = self;

        Url::parse(&format!("{protocol}://{host}:{port}/{path}")).unwrap()
    }
}
impl Into<Url> for EmbedConfig {
    fn into(self) -> Url {
        let EmbedConfig {
            host,
            port,
            path,
            protocol,
            ..
        } = self;

        Url::parse(&format!("{protocol}://{host}:{port}/{path}")).unwrap()
    }
}

impl Into<Url> for UiConfig {
    fn into(self) -> Url {
        let UiConfig {
            host,
            port,
            protocol,
            ..
        } = self;

        Url::parse(&format!("{protocol}://{host}:{port}")).unwrap()
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Config {
            engine:
                EngineConfig {
                    host: engine_host,
                    port: engine_port,
                    index: engine_index,
                    docstore: engine_docstore,
                    conversation_path: engine_conversation_path,
                    query_path: engine_query_path,
                    protocol: engine_protocol,
                },
            embed:
                EmbedConfig {
                    host: embed_host,
                    port: embed_port,
                    model: embed_model,
                    batch_size: embed_batch_size,
                    path: embed_path,
                    protocol: embed_protocol,
                },
            llm:
                LlmConfig {
                    host: llm_host,
                    port: llm_port,
                    path: llm_path,
                    model: llm_model,
                    protocol: llm_protocol,
                },
            ui:
                UiConfig {
                    host: ui_host,
                    port: ui_port,
                    protocol: ui_protocol,
                },
        } = self;

        let engine_index = engine_index.display();
        let engine_docstore = engine_docstore.display();

        let engine_url: Url = self.engine.clone().into();
        let engine_url = engine_url.as_str().yellow();
        let embed_url: Url = self.embed.clone().into();
        let embed_url = embed_url.as_str().yellow();
        let llm_url: Url = self.llm.clone().into();
        let llm_url = llm_url.as_str().yellow();
        let ui_url: Url = self.ui.clone().into();
        let ui_url = ui_url.as_str().yellow();

        write!(
            f,
            "Config:\nEngine running at {engine_url}.\n\tServing conversations on /{engine_conversation_path}.\n\tService queries on /{engine_query_path}.\n\tUsing index at {engine_index}.\n\tUsing docstore at {engine_docstore}.\nUsing embedding service at {embed_url}.\n\tUsing embedder {embed_model} with a batch size of {embed_batch_size}.\nUsing llm service at {llm_url}.\n\tUsing {llm_model}.\nUi running at {ui_url}.",
        )
    }
}
