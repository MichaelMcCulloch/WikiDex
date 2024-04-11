use std::{fmt::Display, path::PathBuf};

use colored::Colorize;
use url::Url;

use crate::{cli_args::ServerArgs, openai::ModelKind};

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) protocol: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) docstore_url: Url,
    pub(crate) system_prompt: String,
    pub(crate) language_model_name: PathBuf,
    pub(crate) language_model_kind: ModelKind,
    pub(crate) embed_model_name: PathBuf,
    pub(crate) embed_url: Url,
    pub(crate) llm_url: Url,
    pub(crate) index_url: Url,
    pub(crate) api_key: Option<String>,
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
            protocol: "http".to_string(),
            host: value.host,
            port: value.port,
            docstore_url: value.docstore_url,
            language_model_name: value.language_model_name,
            language_model_kind: value.language_model_kind,
            embed_url: value.embed_url,
            llm_url: value.llm_url,
            index_url: value.index_url,
            system_prompt: std::fs::read_to_string(value.system_prompt_path).unwrap(),
            api_key: value.api_key,
            embed_model_name: value.embed_model_name,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Config {
            docstore_url: docstore,
            language_model_name: vllm_model,
            embed_model_name: infinity_model,
            embed_url,
            llm_url,
            index_url,
            ..
        } = self;

        let engine_docstore = docstore.as_str().green();
        let index_url = index_url.as_str().green();

        let embed_url = embed_url.as_str().blue();
        let infinity_model = infinity_model.display().to_string().bright_blue();

        let llm_url = llm_url.as_str().blue();
        let vllm_model = vllm_model.display().to_string().bright_blue();

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
Using index at {index_url}.
Using docstore at {engine_docstore}.
Using Infinity embedding service at {embed_url}.
    Using {infinity_model}.
Using vLLM service at {llm_url}.
    Using {vllm_model}."#,
        )
    }
}
