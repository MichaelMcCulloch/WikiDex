#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("features `sqlite` and `postgres` are mutually exclusive");

mod cli_args;
mod config;
mod embedding_client;
mod llm_client;
use {
    async_openai::{config::OpenAIConfig, Client},
    clap::Parser,
    cli_args::{Cli, Commands},
    embedding_client::EmbeddingClient,
    futures::FutureExt,
    std::{ops::DerefMut, sync::Arc, time::Duration},
    tera::Tera,
    tokio::{sync::RwLock, time::sleep},
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
    actix_web::rt,
    config::server::Config as ServerConfig,
    docstore::{Docstore, DocumentStoreImpl},
    index::FaceIndex,
    inference::Engine,
    llm_client::{LlmClient, LlmClientImpl, ModelEndpoint, OpenAiInstructClient, TritonClient},
    server::run_server,
    trtllm::triton::grpc_inference_service_client::GrpcInferenceServiceClient,
};

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
            let system_runner = tokio::runtime::Runtime::new().unwrap();

            log::info!("\n{config}");

            let openai_config = OpenAIConfig::new().with_api_base(config.embed_url.as_ref());
            let open_ai_client: Client<OpenAIConfig> = Client::with_config(openai_config);
            let embedding_client = EmbeddingClient::new(
                open_ai_client,
                config.embed_name.to_string_lossy().to_string(),
            );

            let pipeline = PipelineProcessor;

            system_runner
                .block_on(pipeline.process(
                    &multi_progress,
                    config.wiki_xml,
                    config.output_directory,
                    embedding_client,
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

            let system_prompt = Arc::new(RwLock::new(
                Tera::new(config.system_prompt_template_path.to_str().unwrap()).unwrap(),
            ));
            let tera = system_prompt.clone();
            let llm_client = match config.llm_endpoint {
                ModelEndpoint::Triton => {
                    let client = system_runner.block_on(GrpcInferenceServiceClient::connect(
                        String::from(config.llm_url.as_ref()),
                    ))?;

                    LlmClientImpl::Triton(LlmClient::<TritonClient>::new(client, tera))
                }
                ModelEndpoint::OpenAi => {
                    let openai_config = OpenAIConfig::new().with_api_base(config.llm_url);
                    let open_ai_client = Client::with_config(openai_config);
                    let client = OpenAiInstructClient::new(
                        open_ai_client,
                        config.llm_name.display().to_string(),
                    );
                    let openai_client = system_runner
                        .block_on(LlmClient::<OpenAiInstructClient>::new(client, tera))?;

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

            let engine =
                system_runner.block_on(Engine::new(index, embed_client, llm_client, docstore));

            let run_server = run_server(engine, config.host, config.port);
            let server = run_server?;

            let tera = system_prompt.clone();
            system_runner
                .block_on(async {
                    actix_web::rt::spawn(async move {
                        loop {
                            sleep(Duration::from_secs(2)).await;
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
                    server.await
                })
                .map_err(anyhow::Error::from)
        }
    }
}
