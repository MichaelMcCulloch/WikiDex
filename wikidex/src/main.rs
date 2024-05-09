#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("features `sqlite` and `postgres` are mutually exclusive");

mod cli_args;
mod config;
mod embedding_client;
mod llm_client;
use std::{ops::DerefMut, time::Duration};

use futures::FutureExt;

use {
    async_openai::{config::OpenAIConfig, Client},
    clap::Parser,
    cli_args::{Cli, Commands},
    embedding_client::EmbeddingClient,
};

#[cfg(test)]
mod test_data;

#[cfg(feature = "ingest")]
mod ingest;
#[cfg(feature = "ingest")]
use {
    config::ingest::Config as IngestConfig, indicatif::MultiProgress,
    indicatif_log_bridge::LogWrapper, ingest::pipeline::PipelineProcessor,
};

#[cfg(feature = "server")]
use {std::sync::Arc, tera::Tera, tokio::sync::RwLock};
#[cfg(feature = "server")]
mod docstore;
#[cfg(feature = "server")]
mod formatter;
#[cfg(feature = "server")]
mod index;
#[cfg(feature = "server")]
mod inference;
#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
use {
    config::server::Config as ServerConfig,
    docstore::{Docstore, DocumentStoreImpl},
    index::FaceIndex,
    inference::Engine,
    llm_client::{LlmClient, LlmClientImpl, ModelEndpoint, OpenAiInstructClient, TritonClient},
    server::run_server,
    trtllm::triton::grpc_inference_service_client::GrpcInferenceServiceClient,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

            log::info!("\n{config}");

            let openai_config = OpenAIConfig::new().with_api_base(config.embed_url.as_ref());
            let open_ai_client: Client<OpenAIConfig> = Client::with_config(openai_config);
            let embedding_client = EmbeddingClient::new(
                open_ai_client,
                config.embed_name.to_string_lossy().to_string(),
            );

            let pipeline = PipelineProcessor;

            pipeline
                .process(
                    &multi_progress,
                    config.wiki_xml,
                    config.output_directory,
                    embedding_client,
                )
                .await
                .map_err(anyhow::Error::from)?;
            Ok(())
        }

        #[cfg(feature = "server")]
        Commands::Server(server_args) => {
            env_logger::init();
            let config = ServerConfig::from(server_args);

            log::info!("\n{config}");

            let docstore = match config.docstore_url.scheme() {
                #[cfg(feature = "sqlite")]
                "sqlite" => {
                    let docstore =
                        Docstore::<sqlx::Sqlite>::new(&config.docstore_url, &config.redis_url)
                            .await?;
                    DocumentStoreImpl::Sqlite(docstore)
                }
                #[cfg(feature = "postgres")]
                "postgres" => {
                    let docstore =
                        Docstore::<sqlx::Postgres>::new(&config.docstore_url, &config.redis_url)
                            .await?;

                    DocumentStoreImpl::Postgres(docstore)
                }
                _ => todo!(),
            };

            let index = FaceIndex::new(config.index_url);

            let tera_engine = Arc::new(RwLock::new(
                Tera::new(config.system_prompt_template_path.to_str().unwrap()).unwrap(),
            ));
            let tera = tera_engine.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    tera.write()
                        .map(|mut t| match t.deref_mut().full_reload() {
                            Ok(_) => (),
                            Err(e) => {
                                log::error!("Could Not Reload Template! {e}");
                            }
                        })
                        .await;
                }
            });
            let tera = tera_engine.clone();
            let llm_client = match config.llm_endpoint {
                ModelEndpoint::Triton => {
                    let client =
                        GrpcInferenceServiceClient::connect(String::from(config.llm_url.as_ref()))
                            .await?;

                    LlmClientImpl::Triton(LlmClient::<TritonClient>::new(client, tera))
                }
                ModelEndpoint::OpenAi => {
                    let openai_config = OpenAIConfig::new().with_api_base(config.llm_url);
                    let open_ai_client = Client::with_config(openai_config);
                    let client = OpenAiInstructClient::new(
                        open_ai_client,
                        config.llm_name.display().to_string(),
                    );
                    let openai_client =
                        LlmClient::<OpenAiInstructClient>::new(client, tera).await?;

                    LlmClientImpl::OpenAiInstruct(openai_client)
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

            let engine = Engine::new(index, embed_client, llm_client, docstore).await;

            let run_server = run_server(engine, config.host, config.port);
            let server: actix_web::dev::Server = run_server?;

            server.await.map_err(anyhow::Error::from)
        }
    }
}
