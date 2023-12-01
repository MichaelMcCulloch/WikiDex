use std::{fmt::Display, path::PathBuf};

use colored::Colorize;
use url::Url;

use crate::cli_args::ServerArgs;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) protocol: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) index: PathBuf,
    pub(crate) docstore: PathBuf,
    pub(crate) prompt: String,
    pub(crate) model: PathBuf,
    pub(crate) model_context_length: usize,
    pub(crate) embed_url: Url,
    pub(crate) llm_url: Url,
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
            model: value.model_name,
            model_context_length: value.model_length,
            embed_url: value.embed_url,
            llm_url: value.vllm_url,
            prompt: std::fs::read_to_string(value.prompt_path).unwrap(),
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Config {
            index,
            docstore,
            model,
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
                engine_url.join("conversation").unwrap(),
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
