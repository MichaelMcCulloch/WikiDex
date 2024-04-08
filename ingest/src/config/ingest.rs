use crate::{cli_args::WikipediaIngestArgs, openai::ModelKind};
use colored::Colorize;
use std::{fmt::Display, path::PathBuf};
use url::Url;

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) wiki_xml: PathBuf,
    pub(crate) output_directory: PathBuf,
    pub(crate) embed_url: Url,
    pub(crate) embed_model_name: PathBuf,
    pub(crate) llm_url: Url,
    pub(crate) language_model_name: PathBuf,
    pub(crate) language_model_kind: ModelKind,
    pub(crate) ingest_limit: usize,
    pub(crate) api_key: Option<String>,
}

impl From<WikipediaIngestArgs> for Config {
    fn from(value: WikipediaIngestArgs) -> Self {
        Config {
            wiki_xml: value.wiki_xml,
            output_directory: value.output_directory,
            embed_url: value.embed_url,
            llm_url: value.llm_url,
            language_model_name: value.language_model_name,
            language_model_kind: value.language_model_kind,
            embed_model_name: value.embed_model_name,
            ingest_limit: value.ingest_limit,
            api_key: value.api_key,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Config {
            wiki_xml,
            output_directory,
            embed_url,
            ..
        } = self;

        let wiki_xml = wiki_xml.display();
        let output_directory = output_directory.display();
        let embed_url = embed_url.as_str().yellow();

        write!(
            f,
            "Ingest running.\n\tUsing wikipedia xml dump at {wiki_xml}.\n\tWriting output at {output_directory}.\nUsing Huggingface embedding service at {embed_url}.",
        )
    }
}
