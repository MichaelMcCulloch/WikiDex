mod cli_args;
mod config;
mod docstore;
mod formatter;
mod index;
mod inference;
mod ingest;
mod openai;
mod server;

#[cfg(test)]
mod test_data;

use crate::{
    cli_args::{Cli, Commands},
    index::FaissIndex,
    inference::Engine as InferenceEngine,
    ingest::wikipedia::Engine as WikipediaIngestEngine,
    openai::{ModelKind, OpenAiDelegateBuilder, OpenAiDelegateBuilderArgument},
};
use actix_web::rt;
use clap::Parser;
use docstore::SqliteDocstore;
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use server::run_server;
use std::sync::Mutex;
use url::Url;

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info");

    match Cli::parse().command {
        Commands::Server(server_args) => {
            env_logger::init();
            let config = config::server::Config::from(server_args);
            let system_runner = rt::System::new();

            log::info!("\n{config}");

            let docstore = system_runner.block_on(SqliteDocstore::new(&config.docstore))?;
            let index = FaissIndex::new(&config.index)?;

            let openai_builder =
                OpenAiDelegateBuilder::with_embedding(OpenAiDelegateBuilderArgument::Endpoint(
                    config.embed_url,
                    config.embed_model_name.to_str().unwrap().to_string(),
                ));

            let openai = match config.language_model_kind {
                ModelKind::Instruct => {
                    openai_builder.with_completion(OpenAiDelegateBuilderArgument::Endpoint(
                        config.llm_url,
                        config.language_model_name.to_str().unwrap().to_string(),
                    ))
                }
                ModelKind::Chat => {
                    openai_builder.with_chat(OpenAiDelegateBuilderArgument::Endpoint(
                        config.llm_url,
                        config.language_model_name.to_str().unwrap().to_string(),
                    ))
                }
            };

            let engine =
                InferenceEngine::new(Mutex::new(index), openai, docstore, config.system_prompt);

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
            let system_runner = rt::System::new();

            log::info!("\n{config}");

            let openai_builder = OpenAiDelegateBuilder::with_embedding(
                OpenAiDelegateBuilderArgument::Endpoint(config.embed_url, "".to_string()),
            );
            let openai = openai_builder.with_completion(OpenAiDelegateBuilderArgument::Endpoint(
                Url::parse("").unwrap(),
                "".to_string(),
            ));

            let engine = WikipediaIngestEngine::new(openai, multi_progress, 1024, 128);
            system_runner
                .block_on(engine.ingest_wikipedia(&config.wiki_xml, &config.output_directory))
                .map_err(anyhow::Error::from)?;
            Ok(())
        }
    }
}
