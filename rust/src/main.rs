mod config;
mod docstore;
mod embed;
mod engine;
mod formatter;
mod index;
mod llm;
mod provenance;
mod server;

use clap::Parser;
use config::Config;
use docstore::SqliteDocstore;
use server::run_server;
use std::{path::PathBuf, sync::Mutex};
use url::Url;

use crate::{embed::Embedder, engine::Engine, index::FaissIndex, llm::vllm::VllmService};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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
    #[arg(short, long, default_value_t = Url::parse("http://vllm:5050").unwrap())]
    vllm_url: Url,
    #[arg(short = 'm', long)]
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

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = Args::parse();

    let config = Config::from(args);

    log::info!("\n{config}");

    let embedder = Embedder::new(config.embed_url)?;
    let docstore = SqliteDocstore::new(&config.docstore).await?;
    let index = FaissIndex::new(&config.index)?;
    let llm = VllmService::new(config.llm_url, config.model.to_str().unwrap().to_string())?;

    let engine = Engine::new(Mutex::new(index), embedder, docstore, llm);

    let server = run_server(engine, config.host, config.port)?;
    server.await.map_err(anyhow::Error::from)
}
