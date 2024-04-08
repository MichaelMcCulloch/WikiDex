use std::path::PathBuf;

use clap::{Parser, Subcommand};
use url::Url;

use crate::openai::ModelKind;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}
#[derive(Subcommand)]
pub(crate) enum Commands {
    Wikipedia(WikipediaIngestArgs),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct WikipediaIngestArgs {
    #[arg(long)]
    pub(crate) wiki_xml: PathBuf,
    #[arg(long)]
    pub(crate) output_directory: PathBuf,
    #[arg(long)]
    pub(crate) api_key: Option<String>,
    #[arg(long)]
    pub(crate) embed_url: Url,
    #[arg(long)]
    pub(crate) embed_model_name: PathBuf,
    #[arg(long)]
    pub(crate) llm_url: Url,
    #[arg(long)]
    pub(crate) language_model_name: PathBuf,
    #[arg(long)]
    pub(crate) language_model_kind: ModelKind,
    #[arg(long, default_value_t = 0)]
    pub(crate) ingest_limit: usize,
}
