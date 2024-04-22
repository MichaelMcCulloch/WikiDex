#[cfg(test)]
mod test_data;

mod cli_args;
mod config;
mod embedding_client;
mod llm_client;

#[cfg(feature = "server")]
mod docstore;
#[cfg(feature = "server")]
mod formatter;
#[cfg(feature = "server")]
mod index;
#[cfg(feature = "server")]
mod inference;
#[cfg(feature = "ingest")]
mod ingest;
#[cfg(feature = "server")]
mod server;

use crate::ingest::plain_text::PlainTextProcessor;
#[cfg(feature = "ingest")]
use crate::ingest::wikipedia::Engine as WikipediaIngestEngine;

use crate::llm_client::ModelEndpoint;
use crate::llm_client::OpenAiInstructClient;
use crate::llm_client::TritonClient;

use actix_web::rt;
use async_openai::{config::OpenAIConfig, Client};
use cli_args::Commands;
#[cfg(feature = "server")]
use docstore::Docstore;

#[cfg(feature = "ingest")]
use indicatif::MultiProgress;
#[cfg(feature = "ingest")]
use indicatif_log_bridge::LogWrapper;

use crate::{
    cli_args::Cli,
    embedding_client::EmbeddingClient,
    llm_client::{LlmClient, LlmClientImpl},
};
#[cfg(feature = "server")]
use crate::{docstore::DocumentStoreImpl, index::FaceIndex, inference::Engine, server::run_server};

#[cfg(feature = "ingest")]
use config::ingest::Config as IngestConfig;
#[cfg(feature = "server")]
use config::server::Config as ServerConfig;

use clap::Parser;

#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("features `sqlite` and `postgres` are mutually exclusive");
fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        #[cfg(feature = "ingest")]
        Commands::Wikipedia(ingest_args) => {
            let logger =
                env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                    .build();

            let multi_progress = MultiProgress::new();

            LogWrapper::new(multi_progress.clone(), logger)
                .try_init()
                .unwrap();

            let config = IngestConfig::from(ingest_args);
            let system_runner = rt::System::new();

            log::info!("\n{config}");

            let llm_client = match config.llm_endpoint {
                ModelEndpoint::Triton => {
                    let triton_client = system_runner
                        .block_on(LlmClient::<TritonClient>::new(config.llm_url.as_str()))?;

                    LlmClientImpl::Triton(triton_client)
                }
                ModelEndpoint::OpenAi => {
                    let triton_client =
                        system_runner.block_on(LlmClient::<OpenAiInstructClient>::new(
                            config.llm_url.clone(), // Clone here because temporary use below
                            config.llm_name.to_str().unwrap(),
                        ))?;

                    LlmClientImpl::OpenAiInstruct(triton_client)
                }
            };

            let embed_client = match config.embed_endpoint {
                ModelEndpoint::Triton => todo!(),
                ModelEndpoint::OpenAi => {
                    let openai_config =
                        OpenAIConfig::new().with_api_base(config.embed_url.as_ref());
                    let open_ai_client: Client<OpenAIConfig> = Client::with_config(openai_config);
                    EmbeddingClient::new(
                        open_ai_client,
                        config.embed_name.to_string_lossy().to_string(),
                    )
                }
            };
            let _plaintext = system_runner.block_on(PlainTextProcessor::new(
                config.nebula_url,
                &config.nebula_user,
                &config.nebula_pass,
            ))?;
            let engine =
                WikipediaIngestEngine::new(llm_client, embed_client, multi_progress, 1024, 128);
            system_runner
                .block_on(engine.ingest_wikipedia(
                    &config.wiki_xml,
                    &config.output_directory,
                    config.ingest_limit,
                ))
                .map_err(anyhow::Error::from)?;
            Ok(())
        }

        #[cfg(feature = "server")]
        Commands::Server(server_args) => {
            env_logger::init();
            let config = ServerConfig::from(server_args);
            let system_runner = rt::System::new();

            log::info!("\n{config}");

            let docstore = match config.docstore_url.scheme() {
                #[cfg(feature = "sqlite")]
                "sqlite" => {
                    let docstore = system_runner.block_on(Docstore::<sqlx::Sqlite>::new(
                        &config.docstore_url,
                        &config.redis_url,
                    ))?;

                    DocumentStoreImpl::Sqlite(docstore)
                }
                #[cfg(feature = "postgres")]
                "postgres" => {
                    let docstore = system_runner.block_on(Docstore::<sqlx::Postgres>::new(
                        &config.docstore_url,
                        &config.redis_url,
                    ))?;

                    DocumentStoreImpl::Postgres(docstore)
                }
                _ => todo!(),
            };

            let index = FaceIndex::new(config.index_url);

            let llm_client = match config.llm_endpoint {
                ModelEndpoint::Triton => {
                    let triton_client = system_runner
                        .block_on(LlmClient::<TritonClient>::new(config.llm_url.as_str()))?;

                    LlmClientImpl::Triton(triton_client)
                }
                ModelEndpoint::OpenAi => {
                    let triton_client =
                        system_runner.block_on(LlmClient::<OpenAiInstructClient>::new(
                            config.llm_url.clone(), // Clone here because temporary use below
                            config.llm_name.to_str().unwrap(),
                        ))?;

                    LlmClientImpl::OpenAiInstruct(triton_client)
                }
            };

            let embed_client = match config.embed_endpoint {
                ModelEndpoint::Triton => todo!(),
                ModelEndpoint::OpenAi => {
                    let openai_config =
                        OpenAIConfig::new().with_api_base(config.embed_url.as_ref());
                    let open_ai_client: Client<OpenAIConfig> = Client::with_config(openai_config);
                    EmbeddingClient::new(
                        open_ai_client,
                        config.embed_name.to_string_lossy().to_string(),
                    )
                }
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
