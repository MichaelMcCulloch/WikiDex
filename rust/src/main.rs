mod cli_args;
mod config;
mod docstore;
mod embed;
mod formatter;
mod index;
mod inference;
mod ingest;
mod llm;
mod server;

use crate::{
    cli_args::{Cli, Commands},
    embed::Embedder,
    index::FaissIndex,
    inference::Engine as InferenceEngine,
    ingest::wikipedia::Engine as WikipediaIngestEngine,
    llm::OpenAiService,
};
use clap::Parser;
use docstore::SqliteDocstore;
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use ingest::wikipedia::Ingest;
use server::run_server;
use std::sync::Mutex;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info");

    match Cli::parse().command {
        Commands::Server(server_args) => {
            env_logger::init();
            let config = config::server::Config::from(server_args);

            log::info!("\n{config}");

            let embedder: Embedder = Embedder::new(config.embed_url)?;
            let docstore = SqliteDocstore::new(&config.docstore).await?;
            let index = FaissIndex::new(&config.index)?;
            let llm = OpenAiService::new(
                config.llm_url,
                config.model.to_str().unwrap().to_string(),
                config.model_context_length,
            );

            let engine = InferenceEngine::new(Mutex::new(index), embedder, docstore, llm);

            let server = run_server(engine, config.host, config.port)?;
            server.await.map_err(anyhow::Error::from)
        }
        Commands::Wikipedia(ingest_args) => {
            let logger =
                env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                    .build();

            let multi_progress = MultiProgress::new();

            LogWrapper::new(multi_progress.clone(), logger)
                .try_init()
                .unwrap();

            let config = config::ingest::Config::from(ingest_args);

            log::info!("\n{config}");

            let embedder: Embedder = Embedder::new(config.embed_url)?;
            let llm = OpenAiService::new(
                config.llm_url,
                config.model.to_str().unwrap().to_string(),
                config.model_context_length,
            );

            llm.wait_for_service().await?;
            let engine = WikipediaIngestEngine::new(embedder, llm, multi_progress);

            engine
                .ingest_wikipedia(&config.wiki_xml, &config.output_directory)
                .await?;
            Ok(())
        }
    }
}
