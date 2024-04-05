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
    #[cfg(feature = "server")]
    Server(ServerArgs),
    #[cfg(feature = "ingest")]
    Wikipedia(WikipediaIngestArgs),
    #[cfg(feature = "breeder")]
    Breed(BreederArgs),
}

#[cfg(feature = "breeder")]
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct BreederArgs {
    #[arg(long)]
    pub(crate) index: PathBuf,
    #[arg(long)]
    pub(crate) docstore: PathBuf,
    #[arg(long)]
    pub(crate) thinking_styles_db: PathBuf,
    #[arg(long)]
    pub(crate) mutation_prompts_db: PathBuf,
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
    pub(crate) index_url: Url,
    #[arg(long)]
    pub(crate) language_model_name: PathBuf,
    #[arg(long)]
    pub(crate) language_model_kind: ModelKind,
    #[arg(long)]
    pub(crate) generation_limit: usize,
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

#[cfg(feature = "server")]
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct ServerArgs {
    #[arg( long, default_value_t = String::from("0.0.0.0"))]
    pub(crate) host: String,
    #[arg(long, default_value_t = 5000)]
    pub(crate) port: u16,
    #[arg(long)]
    pub(crate) index: PathBuf,
    #[arg(long)]
    pub(crate) docstore: PathBuf,
    #[arg(long)]
    pub(crate) system_prompt_path: PathBuf,
    #[arg(long)]
    pub(crate) api_key: Option<String>,
    #[arg(long)]
    pub(crate) embed_url: Url,
    #[arg(long)]
    pub(crate) embed_model_name: PathBuf,
    #[arg(long)]
    pub(crate) llm_url: Url,
    #[arg(long)]
    pub(crate) index_url: Url,
    #[arg(long)]
    pub(crate) language_model_name: PathBuf,
    #[arg(long)]
    pub(crate) language_model_kind: ModelKind,
}