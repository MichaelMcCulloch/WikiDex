mod cli_args;
mod config;
mod index;
mod ingest;
mod openai;

#[cfg(test)]
mod test_data;

use crate::ingest::wikipedia::Engine as WikipediaIngestEngine;
use crate::{
    cli_args::{Cli, Commands},
    openai::{ModelKind, OpenAiDelegateBuilder, OpenAiDelegateBuilderArgument},
};
use actix_web::rt;
use clap::Parser;
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
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
                    config.api_key.clone(),
                    config.embed_model_name.to_str().unwrap().to_string(),
                ));
            let openai = match config.language_model_kind {
                ModelKind::Instruct => {
                    openai_builder.with_instruct(OpenAiDelegateBuilderArgument::Endpoint(
                        config.llm_url,
                        config.api_key,
                        config.language_model_name.to_str().unwrap().to_string(),
                    ))
                }
                ModelKind::Chat => {
                    openai_builder.with_chat(OpenAiDelegateBuilderArgument::Endpoint(
                        config.llm_url,
                        config.api_key,
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
    }
}
