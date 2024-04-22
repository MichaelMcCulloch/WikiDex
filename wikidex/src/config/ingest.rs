use crate::{
    cli_args::WikipediaIngestArgs,
    llm_client::{ModelEndpoint, ModelKind},
};
use colored::Colorize;
use std::{fmt::Display, path::PathBuf};
use url::Url;

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) wiki_xml: PathBuf,
    pub(crate) output_directory: PathBuf,
    pub(crate) llm_kind: ModelKind,
    pub(crate) llm_name: PathBuf,
    pub(crate) llm_endpoint: ModelEndpoint,
    pub(crate) llm_url: Url,
    pub(crate) embed_name: PathBuf,
    pub(crate) embed_endpoint: ModelEndpoint,
    pub(crate) embed_url: Url,
    pub(crate) ingest_limit: usize,
    pub(crate) nebula_url: Url,
    pub(crate) nebula_user: String,
    pub(crate) nebula_pass: String,
    pub(crate) api_key: Option<String>,
}

impl From<WikipediaIngestArgs> for Config {
    fn from(value: WikipediaIngestArgs) -> Self {
        Config {
            wiki_xml: value.wiki_xml,
            output_directory: value.output_directory,
            ingest_limit: value.ingest_limit,
            api_key: value.api_key,
            llm_kind: value.llm_kind,
            llm_name: value.llm_name,
            llm_endpoint: value.llm_endpoint,
            llm_url: value.llm_url,
            embed_name: value.embed_name,
            embed_endpoint: value.embed_endpoint,
            embed_url: value.embed_url,
            nebula_url: value.nebula_url,
            nebula_user: value.nebula_user,
            nebula_pass: value.nebula_pass,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Config {
            wiki_xml,
            output_directory,
            embed_url,
            llm_kind: _,
            llm_name,
            llm_endpoint,
            llm_url,
            embed_name,
            embed_endpoint,
            ingest_limit,
            api_key: _,
            ..
        } = self;

        let wiki_xml = wiki_xml.display();
        let output_directory = output_directory.display();
        let embed_url = embed_url.as_str().blue();
        let embed_endpoint = format!("{embed_endpoint}").as_str().blue();
        let embed_name = embed_name.display().to_string().bright_blue();

        let llm_url = llm_url.as_str().blue();
        let llm_endpoint = format!("{llm_endpoint}").as_str().blue();
        let llm_model = llm_name.display().to_string().bright_blue();

        write!(
            f,
            r###"Ingest running.
    Using wikipedia xml dump at {wiki_xml}.
    Writing output at {output_directory}.
    Maximum {ingest_limit} articles.
Using {llm_endpoint} llm service at {llm_url}.
    Using {llm_model}.
Using {embed_endpoint} embed service at {embed_url}.
    Using {embed_name}.
"###,
        )
    }
}
