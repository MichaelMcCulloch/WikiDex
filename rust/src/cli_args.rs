use std::path::PathBuf;

use clap::{ArgGroup, Parser, Subcommand};
use url::Url;

use crate::llm::ModelKind;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    Server(ServerArgs),
    Wikipedia(WikipediaIngestArgs),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct WikipediaIngestArgs {
    #[arg(short, long)]
    pub(crate) wiki_xml: PathBuf,
    #[arg(short, long)]
    pub(crate) output_directory: PathBuf,

    #[arg(short, long, default_value_t = Url::parse("http://embeddings:9000").unwrap())]
    pub(crate) embed_url: Url,
    #[arg(short = 'm', long)]
    pub(crate) embed_model_name: PathBuf,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(ArgGroup::new("endpoint").args(&["vllm_url", "openai_key"]).multiple(false).required(true)))]
pub(crate) struct ServerArgs {
    #[arg(short = 'a' , long, default_value_t = String::from("0.0.0.0"))]
    pub(crate) host: String,
    #[arg(short, long, default_value_t = 5000)]
    pub(crate) port: u16,
    #[arg(short, long)]
    pub(crate) index: PathBuf,
    #[arg(short, long)]
    pub(crate) docstore: PathBuf,
    #[arg(short, long)]
    pub(crate) system_prompt_path: PathBuf,
    #[arg(short, long, default_value_t = Url::parse("http://embeddings:9000/v1").unwrap())]
    pub(crate) embed_url: Url,
    #[arg(short, long, default_value_t = Url::parse("http://vllm:5050/v1").unwrap(), group = "endpoint")]
    pub(crate) vllm_url: Url,
    #[arg(short, long, group = "endpoint")]
    pub(crate) openai_key: Option<String>,
    #[arg(short, long)]
    pub(crate) language_model_name: PathBuf,
    #[arg(short = 'k', long)]
    pub(crate) language_model_kind: ModelKind,
    #[arg(short = 'm', long)]
    pub(crate) embed_model_name: PathBuf,
}
