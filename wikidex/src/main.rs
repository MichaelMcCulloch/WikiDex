#[cfg(test)]
mod test_data;

mod cli_args;
mod config;
mod docstore;
mod embedding_client;
mod formatter;
mod index;
mod inference;
mod llm_client;
mod server;

#[cfg(feature = "openai")]
use crate::llm_client::OpenAiInstructClient;
#[cfg(feature = "triton")]
use crate::llm_client::TritonClient;

use actix_web::rt;
use async_openai::{config::OpenAIConfig, Client};
use cli_args::Commands;
use docstore::Docstore;

#[cfg(feature = "postgres")]
use sqlx::Postgres;
#[cfg(feature = "sqlite")]
use sqlx::Sqlite;

use crate::{
    cli_args::Cli,
    config::server::Config,
    docstore::DocumentStoreKind,
    embedding_client::EmbeddingClient,
    index::FaceIndex,
    inference::Engine,
    llm_client::{LlmClient, LlmClientKind},
    server::run_server,
};

use clap::Parser;

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Commands::Server(server_args) => {
            env_logger::init();
            let config = Config::from(server_args);
            let system_runner = rt::System::new();

            log::info!("\n{config}");

            let docstore = match config.docstore_url.scheme() {
                #[cfg(feature = "sqlite")]
                "sqlite" => {
                    let docstore = system_runner.block_on(Docstore::<Sqlite>::new(
                        &config.docstore_url,
                        &config.redis_url,
                    ))?;

                    DocumentStoreKind::Sqlite(docstore)
                }
                #[cfg(feature = "postgres")]
                "postgres" => {
                    let docstore = system_runner.block_on(Docstore::<Postgres>::new(
                        &config.docstore_url,
                        &config.redis_url,
                    ))?;

                    DocumentStoreKind::Postgres(docstore)
                }
                _ => todo!(),
            };

            let index = FaceIndex::new(config.index_url);

            #[cfg(feature = "triton")]
            let _llm_client = {
                let triton_client = system_runner
                    .block_on(LlmClient::<TritonClient>::new(config.triton_url.as_str()))?;

                LlmClientKind::Triton(triton_client)
            };
            #[cfg(feature = "openai")]
            let llm_client = {
                let triton_client =
                    system_runner.block_on(LlmClient::<OpenAiInstructClient>::new(
                        config.openai_url.clone(), // Clone here because temporary use below
                        config.language_model_name.to_str().unwrap(),
                    ))?;

                LlmClientKind::OpenAiInstruct(triton_client)
            };
            let embed_client = {
                let openai_config = OpenAIConfig::new().with_api_base(config.embed_url.as_ref());
                let open_ai_client = Client::with_config(openai_config);
                EmbeddingClient::new(
                    open_ai_client,
                    config.embed_model_name.to_string_lossy().to_string(),
                )
            };

            let engine = Engine::new(
                index,
                embed_client,
                llm_client,
                docstore,
                config.system_prompt,
            );

            let run_server = run_server(engine, config.host, config.port);
            let server = run_server?;
            system_runner.block_on(server).map_err(anyhow::Error::from)
        }
    }
}
