mod breeder;
mod cli_args;
mod config;
mod docstore;
mod formatter;
mod index;
mod openai;

#[cfg(test)]
mod test_data;

use docstore::SqliteDocstore;
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;

use crate::{
    breeder::Engine as PromptBreedingEngine,
    cli_args::{Cli, Commands},
    index::FaceIndex,
    openai::{ModelKind, OpenAiDelegateBuilder, OpenAiDelegateBuilderArgument},
};

use actix_web::rt;
use clap::Parser;
use std::fs;

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
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
            let index = FaceIndex::new(config.index_url);

            let thinking_styles = fs::read_to_string(config.thinking_styles_db)?
                .split('\n')
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
            let mutation_prompts = fs::read_to_string(config.mutation_prompts_db)?
                .split('\n')
                .map(|s| s.to_string())
                .collect::<Vec<_>>();

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

            let engine = PromptBreedingEngine::new(
                index,
                openai,
                docstore,
                thinking_styles,
                mutation_prompts,
            );

            let problem_description =
                "Answer the question with a summary based off the provided documents.";

            let _prompt = system_runner
                .block_on(engine.breed_prompt(problem_description, config.generation_limit))
                .map_err(anyhow::Error::from)?;
            Ok(())
        }
    }
}
