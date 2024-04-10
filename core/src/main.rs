mod cli_args;
mod config;
mod index;
mod openai;

#[cfg(test)]
mod test_data;

mod docstore;
mod formatter;
mod inference;
mod server;

use actix_web::rt;
use cli_args::Commands;
use docstore::Docstore;

use crate::{
    cli_args::Cli,
    index::FaceIndex,
    inference::Engine as InferenceEngine,
    openai::{ModelKind, OpenAiDelegateBuilder, OpenAiDelegateBuilderArgument},
    server::run_server,
};

use clap::Parser;

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Commands::Server(server_args) => {
            env_logger::init();
            let config = config::server::Config::from(server_args);
            let system_runner = rt::System::new();

            log::info!("\n{config}");

            let docstore = system_runner.block_on(Docstore::new(&config.docstore))?;

            let index = FaceIndex::new(config.index_url);

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

            let engine = InferenceEngine::new(index, openai, docstore, config.system_prompt);

            let server = run_server(engine, config.host, config.port)?;
            system_runner.block_on(server).map_err(anyhow::Error::from)
        }
    }
}
