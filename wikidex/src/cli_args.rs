use std::path::PathBuf;

use clap::{Parser, Subcommand};
use url::Url;

use crate::llm_client::{ModelEndpoint, ModelKind};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}
#[derive(Subcommand)]
pub(crate) enum Commands {
    #[cfg(feature = "server")]
    Server(ServerArgs),
    #[cfg(feature = "ingest")]
    Wikipedia(WikipediaIngestArgs),
}

#[cfg(feature = "server")]
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct ServerArgs {
    #[arg( long, default_value_t = String::from("0.0.0.0"))]
    pub(crate) host: String,
    #[arg(long, default_value_t = 5000)]
    pub(crate) port: u16,
    #[arg(long)]
    pub(crate) docstore_url: Url,
    #[arg(long)]
    pub(crate) redis_url: Url,
    #[arg(long)]
    pub(crate) system_prompt_path: PathBuf,
    #[arg(long)]
    pub(crate) api_key: Option<String>,
    #[arg(long)]
    pub(crate) index_url: Url,
    #[arg(long)]
    pub(crate) llm_kind: ModelKind,
    #[arg(long)]
    pub(crate) llm_name: PathBuf,
    #[arg(long)]
    pub(crate) llm_endpoint: ModelEndpoint,
    #[arg(long)]
    pub(crate) llm_url: Url,
    #[arg(long)]
    pub(crate) embed_name: PathBuf,
    #[arg(long)]
    pub(crate) embed_endpoint: ModelEndpoint,
    #[arg(long)]
    pub(crate) embed_url: Url,
}
#[cfg(feature = "ingest")]
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
    pub(crate) llm_kind: ModelKind,
    #[arg(long)]
    pub(crate) llm_name: PathBuf,
    #[arg(long)]
    pub(crate) llm_endpoint: ModelEndpoint,
    #[arg(long)]
    pub(crate) llm_url: Url,
    #[arg(long)]
    pub(crate) embed_name: PathBuf,
    #[arg(long)]
    pub(crate) embed_endpoint: ModelEndpoint,
    #[arg(long)]
    pub(crate) embed_url: Url,
    #[arg(long, default_value_t = 0)]
    pub(crate) ingest_limit: usize,
    #[arg(long)]
    pub(crate) nebula_url: Url,
    #[arg(long)]
    pub(crate) nebula_user: String,
    #[arg(long)]
    pub(crate) nebula_pass: String,
}
