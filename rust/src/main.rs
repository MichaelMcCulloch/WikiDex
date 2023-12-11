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

#[cfg(test)]
mod test_data;

use crate::{
    cli_args::{Cli, Commands},
    embed::r#async::Embedder,
    embed::sync::Embedder as SEmbedder,
    index::FaissIndex,
    inference::Engine as InferenceEngine,
    ingest::wikipedia::Engine as WikipediaIngestEngine,
    llm::AsyncOpenAiService,
};
use actix_web::rt;
use clap::Parser;
use docstore::SqliteDocstore;
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use ingest::wikipedia::Ingest;
use server::run_server;
use std::sync::Mutex;

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info");

    match Cli::parse().command {
        Commands::Server(server_args) => {
            env_logger::init();
            let config = config::server::Config::from(server_args);
            let system_runner = rt::System::new();

            log::info!("\n{config}");

            let embedder = Embedder::new(config.embed_url)?;
            let docstore = system_runner.block_on(SqliteDocstore::new(&config.docstore))?;
            let index = FaissIndex::new(&config.index)?;
            let llm = AsyncOpenAiService::new(
                config.openai_key,
                config.llm_url,
                config.model.to_str().unwrap().to_string(),
            );

            let engine = InferenceEngine::new(
                Mutex::new(index),
                embedder,
                docstore,
                llm,
                config.system_prompt,
            );

            let server = run_server(engine, config.host, config.port)?;
            system_runner.block_on(server).map_err(anyhow::Error::from)
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

            let embedder = SEmbedder::new(config.embed_url)?;

            let engine = WikipediaIngestEngine::new(embedder, multi_progress, 1024, 128);

            engine.ingest_wikipedia(&config.wiki_xml, &config.output_directory)?;
            Ok(())
        }
    }
}
