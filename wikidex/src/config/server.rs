use std::{fmt::Display, path::PathBuf};

use colored::Colorize;
use url::Url;

use crate::cli_args::ServerArgs;

#[derive(Debug)]
pub(crate) struct Config {
    #[cfg(feature = "openai")]
    pub(crate) api_key: Option<String>,
    pub(crate) docstore_url: Url,
    pub(crate) embed_model_name: PathBuf,
    pub(crate) embed_url: Url,
    pub(crate) host: String,
    pub(crate) index_url: Url,
    pub(crate) language_model_name: PathBuf,
    #[cfg(feature = "openai")]
    pub(crate) openai_url: Url,
    #[cfg(feature = "triton")]
    pub(crate) triton_url: Url,
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
            #[cfg(feature = "openai")]
            api_key: value.api_key,
            docstore_url: value.docstore_url,
            embed_model_name: value.embed_model_name,
            embed_url: value.embed_url,
            host: value.host,
            index_url: value.index_url,
            language_model_name: value.language_model_name,
            #[cfg(feature = "openai")]
            openai_url: value.openai_url,
            #[cfg(feature = "triton")]
            triton_url: value.triton_url,
            port: value.port,
            protocol: "http".to_string(),
            redis_url: value.redis_url,
            system_prompt: std::fs::read_to_string(value.system_prompt_path).unwrap(),
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Config {
            docstore_url,
            language_model_name,
            embed_model_name,
            embed_url,
            #[cfg(feature = "openai")]
                openai_url: llm_url,
            #[cfg(feature = "triton")]
                triton_url: llm_url,
            index_url,
            redis_url,
            ..
        } = self;

        let docstore_url = docstore_url.as_str().green();
        let index_url = index_url.as_str().green();
        let redis_url = redis_url.as_str().green();

        let embed_url = embed_url.as_str().blue();
        let infinity_model = embed_model_name.display().to_string().bright_blue();

        let llm_url = llm_url.as_str().blue();
        let vllm_model = language_model_name.display().to_string().bright_blue();

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
Using Infinity embedding service at {embed_url}.
    Using {infinity_model}.
Using vLLM service at {llm_url}.
    Using {vllm_model}."#,
        )
    }
}
