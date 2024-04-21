use std::{fmt::Display, path::PathBuf};

use colored::Colorize;
use url::Url;

use crate::{
    cli_args::ServerArgs,
    llm_client::{ModelEndpoint, ModelKind},
};

#[derive(Debug)]
pub(crate) struct Config {
    // Me
    pub(crate) api_key: Option<String>,
    pub(crate) docstore_url: Url,

    pub(crate) host: String,
    pub(crate) index_url: Url,
    pub(crate) llm_kind: ModelKind,
    pub(crate) llm_name: PathBuf,
    pub(crate) llm_endpoint: ModelEndpoint,
    pub(crate) llm_url: Url,
    pub(crate) embed_name: PathBuf,
    pub(crate) embed_endpoint: ModelEndpoint,
    pub(crate) embed_url: Url,
    pub(crate) port: u16,
    pub(crate) protocol: String,
    pub(crate) redis_url: Url,
    pub(crate) system_prompt: String,
}

pub(crate) trait ConfigUrl {
    fn url(&self) -> Url;
}

impl ConfigUrl for Config {
    fn url(&self) -> Url {
        let Config {
            protocol,
            host,
            port,
            ..
        } = self;

        Url::parse(&format!("{protocol}://{host}:{port}")).unwrap()
    }
}

impl From<ServerArgs> for Config {
    fn from(value: ServerArgs) -> Self {
        Config {
            api_key: value.api_key,
            docstore_url: value.docstore_url,
            host: value.host,
            index_url: value.index_url,
            port: value.port,
            protocol: "http".to_string(),
            redis_url: value.redis_url,
            system_prompt: std::fs::read_to_string(value.system_prompt_path).unwrap(),
            llm_kind: value.llm_kind,
            llm_name: value.llm_name,
            llm_endpoint: value.llm_endpoint,
            llm_url: value.llm_url,
            embed_name: value.embed_name,
            embed_endpoint: value.embed_endpoint,
            embed_url: value.embed_url,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Config {
            docstore_url,
            index_url,
            redis_url,
            api_key: _,
            host: _,
            llm_kind: _,
            llm_name,
            llm_endpoint,
            llm_url,
            embed_name,
            embed_endpoint,
            embed_url,
            port: _,
            protocol: _,
            system_prompt: _,
        } = self;

        let docstore_url = docstore_url.as_str().green();
        let redis_url = redis_url.as_str().green();

        let index_url = index_url.as_str().green();

        let embed_url = embed_url.as_str().blue();
        let embed_endpoint = format!("{embed_endpoint}").as_str().blue();
        let embed_name = embed_name.display().to_string().bright_blue();

        let llm_url = llm_url.as_str().blue();
        let llm_endpoint = format!("{llm_endpoint}").as_str().blue();
        let llm_model = llm_name.display().to_string().bright_blue();

        let engine_url = self.url();
        let [engine_conversation_path, engine_query_path, engine_api_doc_path] = [
            engine_url.join("streaming_conversation").unwrap(),
            engine_url.join("query").unwrap(),
            engine_url.join("api-doc").unwrap(),
        ]
        .map(|url| url.as_str().yellow());

        write!(
            f,
            r#"Engine running.
    Serving conversations on {engine_conversation_path}.
    Service queries on {engine_query_path}.
    Serving OpenAPI documentation on {engine_api_doc_path}.
Using redis at {redis_url}.
Using index at {index_url}.
Using docstore at {docstore_url}.
Using {embed_endpoint} embedding service at {embed_url}.
    Using {embed_name}.
Using {llm_endpoint} service at {llm_url}.
    Using {llm_model}."#,
        )
    }
}
