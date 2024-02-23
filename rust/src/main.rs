mod cli_args;
mod config;
mod index;
mod openai;

use crate::{
    cli_args::{Cli, Commands},
    openai::ModelKind,
    openai::{OpenAiDelegateBuilder, OpenAiDelegateBuilderArgument},
};
use actix_web::rt;
use clap::Parser;

#[cfg(test)]
mod test_data;

#[cfg(feature = "server")]
mod docstore;
#[cfg(feature = "server")]
mod formatter;
#[cfg(feature = "server")]
mod inference;
#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
use crate::{index::FaissIndex, inference::Engine as InferenceEngine};
#[cfg(feature = "server")]
use docstore::SqliteDocstore;
#[cfg(feature = "server")]
use server::run_server;
#[cfg(feature = "server")]
use std::sync::Mutex;

#[cfg(feature = "ingest")]
mod ingest;
#[cfg(feature = "ingest")]
use crate::ingest::wikipedia::Engine as WikipediaIngestEngine;
#[cfg(any(feature = "ingest", feature = "breeder"))]
use indicatif::MultiProgress;
#[cfg(any(feature = "ingest", feature = "breeder"))]
use indicatif_log_bridge::LogWrapper;

// #[cfg(any(feature = "ingest",feature = "breeder"))]

#[cfg(feature = "breeder")]
mod breeder;
#[cfg(feature = "ingest")]
use crate::breeder::Engine as PromptBreedingEngine;

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    match Cli::parse().command {
        #[cfg(feature = "server")]
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
                    openai_builder.with_instruct(OpenAiDelegateBuilderArgument::Endpoint(
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
        #[cfg(feature = "ingest")]
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

            let openai_builder =
                OpenAiDelegateBuilder::with_embedding(OpenAiDelegateBuilderArgument::Endpoint(
                    config.embed_url,
                    config.embed_model_name.to_str().unwrap().to_string(),
                ));
            let openai = match config.language_model_kind {
                ModelKind::Instruct => {
                    openai_builder.with_instruct(OpenAiDelegateBuilderArgument::Endpoint(
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

            let engine = WikipediaIngestEngine::new(openai, multi_progress, 1024, 128);
            system_runner
                .block_on(engine.ingest_wikipedia(
                    &config.wiki_xml,
                    &config.output_directory,
                    config.ingest_limit,
                ))
                .map_err(anyhow::Error::from)?;
            Ok(())
        }
        #[cfg(feature = "breeder")]
        Commands::Breed(breeder_args) => {
            let logger =
                env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                    .build();

            let multi_progress = MultiProgress::new();

            LogWrapper::new(multi_progress.clone(), logger)
                .try_init()
                .unwrap();

            let config = config::breeder::Config::from(breeder_args);
            let system_runner = rt::System::new();

            let docstore = system_runner.block_on(SqliteDocstore::new(&config.docstore))?;
            let index = FaissIndex::new(&config.index)?;

            let openai_builder =
                OpenAiDelegateBuilder::with_embedding(OpenAiDelegateBuilderArgument::Endpoint(
                    config.embed_url,
                    config.embed_model_name.to_str().unwrap().to_string(),
                ));

            let openai = match config.language_model_kind {
                ModelKind::Instruct => {
                    openai_builder.with_instruct(OpenAiDelegateBuilderArgument::Endpoint(
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

            let engine = PromptBreedingEngine::new(Mutex::new(index), openai, docstore);

            let _prompt = system_runner
                .block_on(engine.breed_prompt())
                .map_err(anyhow::Error::from)?;
            todo!()
        }
    }
}
