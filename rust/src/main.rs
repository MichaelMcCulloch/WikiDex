mod cli_args;
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
use std::sync::Mutex;

use crate::{
    cli_args::Args, embed::Embedder, engine::Engine, index::FaissIndex, llm::vllm::VllmService,
};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = Args::parse();

    let config = Config::from(args);

    log::info!("\n{config}");

    let embedder: Embedder = Embedder::new(config.embed_url)?;
    let docstore = SqliteDocstore::new(&config.docstore).await?;
    let index = FaissIndex::new(&config.index)?;
    let llm = VllmService::new(config.llm_url, config.model.to_str().unwrap().to_string())?;

    let engine = Engine::new(Mutex::new(index), embedder, docstore, llm);

    let server = run_server(engine, config.host, config.port)?;
    server.await.map_err(anyhow::Error::from)
}
