use std::{fmt::Display, path::PathBuf};

use colored::Colorize;
use url::Url;

use crate::cli_args::IngestArgs;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) wiki_xml: PathBuf,
    pub(crate) output_directory: PathBuf,
    pub(crate) model: PathBuf,
    pub(crate) embed_url: Url,
    pub(crate) llm_url: Url,
}
impl From<IngestArgs> for Config {
    fn from(value: IngestArgs) -> Self {
        Config {
            wiki_xml: value.wiki_xml,
            output_directory: value.output_directory,
            model: value.model_name,
            embed_url: value.embed_url,
            llm_url: value.vllm_url,
        }
    }
}
impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Config {
            wiki_xml,
            output_directory,
            model,
            embed_url,
            llm_url,
        } = self;

        let wiki_xml = wiki_xml.display();
        let output_directory = output_directory.display();

        let model = model.display();

        let [embed_url, llm_url] =
            [embed_url.clone(), llm_url.clone()].map(|url| url.as_str().yellow());

        write!(
            f,
            "Ingest running.\n\tUsing wikipedia xml dump at {wiki_xml}.\n\tWriting output at {output_directory}.\nUsing Huggingface embedding service at {embed_url}.\nUsing vLLM service at {llm_url}.\n\tUsing {model}.",
        )
    }
}
