use std::{fmt::Display, path::PathBuf};

use colored::Colorize;
use url::Url;

use crate::cli_args::WikipediaIngestArgs;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) wiki_xml: PathBuf,
    pub(crate) output_directory: PathBuf,
    pub(crate) embed_url: Url,
}
impl From<WikipediaIngestArgs> for Config {
    fn from(value: WikipediaIngestArgs) -> Self {
        Config {
            wiki_xml: value.wiki_xml,
            output_directory: value.output_directory,
            embed_url: value.embed_url,
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
