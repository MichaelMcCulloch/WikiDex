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
    Breed(BreederArgs),
}

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
