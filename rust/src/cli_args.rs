use std::path::PathBuf;

use clap::Parser;
use url::Url;

use crate::config::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(short = 'a' , long, default_value_t = String::from("0.0.0.0"))]
    host: String,
    #[arg(short, long, default_value_t = 5000)]
    port: u16,
    #[arg(short, long)]
    index: PathBuf,
    #[arg(short, long)]
    docstore: PathBuf,

    #[arg(short, long, default_value_t = Url::parse("http://embeddings:9000").unwrap())]
    embed_url: Url,
    #[arg(short, long, default_value_t = Url::parse("http://vllm:5050/v1").unwrap())]
    vllm_url: Url,
    #[arg(short, long)]
    model_name: PathBuf,
}

impl From<Args> for Config {
    fn from(value: Args) -> Self {
        Config {
            protocol: "http".to_string(),
            host: value.host,
            port: value.port,
            index: value.index,
            docstore: value.docstore,
            model: value.model_name,
            embed_url: value.embed_url,
            llm_url: value.vllm_url,
        }
    }
}
