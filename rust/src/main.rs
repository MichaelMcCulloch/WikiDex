mod cli_args;
mod config;
mod docstore;
mod embed;
mod formatter;
mod index;
mod ingest;
mod llm;
mod provenance;
mod server;

use clap::Parser;
use cli_args::Commands;
use docstore::SqliteDocstore;
use server::run_server;
use std::sync::Mutex;

use crate::{cli_args::Cli, embed::Embedder, index::FaissIndex, llm::vllm::VllmService};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    match Cli::parse().command {
        Commands::Server(server_args) => {
            let config = config::server::Config::from(server_args);

            log::info!("\n{config}");

            let embedder: Embedder = Embedder::new(config.embed_url)?;
            let docstore = SqliteDocstore::new(&config.docstore).await?;
            let index = FaissIndex::new(&config.index)?;
            let llm = VllmService::new(config.llm_url, config.model.to_str().unwrap().to_string())?;

            let engine = server::engine::Engine::new(Mutex::new(index), embedder, docstore, llm);

            let server = run_server(engine, config.host, config.port)?;
            server.await.map_err(anyhow::Error::from)
        }
        Commands::Ingest(ingest_args) => {
            let config = config::ingest::Config::from(ingest_args);

            let embedder: Embedder = Embedder::new(config.embed_url)?;
            let llm = VllmService::new(config.llm_url, config.model.to_str().unwrap().to_string())?;

            Ok(())
        }
    }
}
