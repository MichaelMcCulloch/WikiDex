mod config;
mod docstore;
mod embed;
mod engine;
mod formatter;
mod index;
mod llm;
mod provenance;
mod server;

use config::Config;
use docstore::SqliteDocstore;
use server::run_server;
use std::sync::Mutex;

use crate::{embed::Embedder, engine::Engine, index::FaissIndex, llm::Llm};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let toml_str = std::fs::read_to_string("/home/michael/Development/oracle/Config.TOML")?;
    let config: Config = toml::from_str(&toml_str)?;

    log::info!("\n{config}");

    let embedder = Embedder::new(config.embed)?;
    let docstore = SqliteDocstore::new(&config.engine.docstore).await?;
    let index = FaissIndex::new(&config.engine.index)?;
    let llm = Llm::new(config.llm)?;

    let engine = Engine::new(Mutex::new(index), embedder, docstore, llm);

    let server = run_server(engine, config.engine)?;
    server.await.map_err(anyhow::Error::from)
}
