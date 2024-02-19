use std::{fmt::Display, path::PathBuf};

use colored::Colorize;
use url::Url;

#[cfg(feature = "server")]
use crate::{cli_args::ServerArgs, openai::ModelKind};

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) protocol: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) index: PathBuf,
    pub(crate) docstore: PathBuf,
    pub(crate) system_prompt: String,
    pub(crate) language_model_name: PathBuf,
    pub(crate) language_model_kind: ModelKind,
    pub(crate) embed_model_name: PathBuf,
    pub(crate) embed_url: Url,
    pub(crate) llm_url: Url,
    pub(crate) openai_key: Option<String>,
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
            index: value.index,
            docstore: value.docstore,
            language_model_name: value.language_model_name,
            language_model_kind: value.language_model_kind,
            embed_url: value.embed_url,
            llm_url: value.llm_url,
            system_prompt: std::fs::read_to_string(value.system_prompt_path).unwrap(),
            openai_key: value.openai_key,
            embed_model_name: value.embed_model_name,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Config {
            index,
            docstore,
            language_model_name: model,
            embed_url,
            llm_url,
            ..
        } = self;

        let engine_index = index.display();
        let engine_docstore = docstore.display();
        let engine_url = self.url();

        let model = model.display();

        let [engine_conversation_path, engine_query_path, engine_api_doc_path, embed_url, llm_url] =
            [
                engine_url.join("streaming_conversation").unwrap(),
                engine_url.join("query").unwrap(),
                engine_url.join("api-doc").unwrap(),
                embed_url.clone(),
                llm_url.clone(),
            ]
            .map(|url| url.as_str().yellow());

        write!(
            f,
            "Engine running.\n\tServing conversations on {engine_conversation_path}.\n\tService queries on {engine_query_path}.\n\tServing OpenAPI documentation on {engine_api_doc_path}.\n\tUsing index at {engine_index}.\n\tUsing docstore at {engine_docstore}.\nUsing Huggingface embedding service at {embed_url}.\nUsing vLLM service at {llm_url}.\n\tUsing {model}.",
        )
    }
}
