use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{Sqlite, SqliteConnectOptions, SqlitePoolOptions};

use tokio::sync::mpsc::unbounded_channel;

use crate::embedding_client::EmbeddingClient;
use crate::ingest::pipeline::steps::WikipediaDumpReader;
use crate::ingest::pipeline::{error::PipelineError, steps::PipelineStep};

use super::document::{DocumentCompressed, DocumentHeading};
use super::error::Sql;
use super::steps::{Batcher, Embedding, SqliteWriter};

use super::{
    steps::{Compressor, WikipediaHeadingSplitter, WikipediaMarkdownParser},
    wikipedia::WikiMarkupProcessor,
};

pub(crate) struct PipelineProcessor;

impl PipelineProcessor {
    pub(crate) async fn process(
        &self,
        multi_progress: &MultiProgress,
        wiki_xml: PathBuf,
        database_output_directory: PathBuf,
        embedding_client: EmbeddingClient,
    ) -> Result<(), PipelineError> {
        let docstore_path = {
            let mut p = database_output_directory.clone();
            p.push("wikipedia_docstore.sqlite");
            p.display().to_string()
        };
        let index_path = {
            let mut p = database_output_directory.clone();
            p.push("wikipedia_index.sqlite");
            p.display().to_string()
        };

        if !Sqlite::database_exists(&docstore_path)
            .await
            .map_err(Sql::Sql)?
        {
            Sqlite::create_database(&docstore_path)
                .await
                .map_err(Sql::Sql)?;
        }
        if !Sqlite::database_exists(&index_path)
            .await
            .map_err(Sql::Sql)?
        {
            Sqlite::create_database(&index_path)
                .await
                .map_err(Sql::Sql)?;
        }

        let options = SqliteConnectOptions::new();

        let options = options.pragma("locking_mode", "EXCLUSIVE");
        let options = options.pragma("journal_mode", "WAL");
        let options = options.pragma("synchronous", "normal");
        let options = options.pragma("temp_store", "memory");
        let options = options.pragma("mmap_size", "30000000");
        let options = options.create_if_missing(true);
        let docstore_option = options.clone().filename(docstore_path);
        let index_options = options.clone().filename(index_path);

        let docstore_pool = SqlitePoolOptions::new()
            .acquire_timeout(Duration::from_secs(10000))
            .max_connections(1)
            .connect_with(docstore_option)
            .await
            .map_err(Sql::Sql)?;

        let index_pool = SqlitePoolOptions::new()
            .acquire_timeout(Duration::from_secs(10000))
            .max_connections(1)
            .connect_with(index_options)
            .await
            .map_err(Sql::Sql)?;

        let reader = WikipediaDumpReader::new(0);
        let parser = WikipediaMarkdownParser::new(WikiMarkupProcessor);
        let wikisplitter = WikipediaHeadingSplitter::default();
        let step_batch_10240 = Batcher::<10240, DocumentCompressed>::default();
        let step_batch_512 = Batcher::<512, DocumentHeading>::default();
        let step_embed = Embedding::new(embedding_client);
        let step_compress = Compressor;
        let step_save = SqliteWriter::new(docstore_pool, index_pool).await?;

        let reader_progress = new_progress_bar(multi_progress, 0);
        let parser_progress = new_progress_bar(multi_progress, 0);
        let wikisplitter_progress = new_progress_bar(multi_progress, 0);
        let embedding_batcher_progress = new_progress_bar(multi_progress, 0);
        let embedding_progress = new_progress_bar(multi_progress, 0);
        let compressor_progress = new_progress_bar(multi_progress, 0);
        let docstore_batcher_progress = new_progress_bar(multi_progress, 0);
        let writter_progress = new_progress_bar(multi_progress, 0);
        let docstore_completed_progress = new_progress_bar(multi_progress, 0);

        docstore_completed_progress.set_message("Docstore");

        let (t, rx_pathbuf) = unbounded_channel::<PathBuf>();
        let mut rx_reader = reader
            .link(
                rx_pathbuf,
                reader_progress.clone(),
                vec![parser_progress.clone()],
            )
            .await?;
        let mut rx_parser = parser
            .link(
                rx_reader.pop().unwrap(),
                parser_progress.clone(),
                vec![wikisplitter_progress.clone()],
            )
            .await?;
        let mut rx_heading_split = wikisplitter
            .link(
                rx_parser.pop().unwrap(),
                wikisplitter_progress.clone(),
                vec![embedding_batcher_progress.clone()],
            )
            .await?;

        let mut rx_embedding_batcher = step_batch_512
            .link(
                rx_heading_split.pop().unwrap(),
                embedding_batcher_progress.clone(),
                vec![embedding_progress.clone()],
            )
            .await?;
        let mut rx_embedder = step_embed
            .link(
                rx_embedding_batcher.pop().unwrap(),
                embedding_progress,
                vec![compressor_progress.clone()],
            )
            .await?;
        let mut rx_compressor = step_compress
            .link(
                rx_embedder.pop().unwrap(),
                compressor_progress.clone(),
                vec![docstore_batcher_progress.clone()],
            )
            .await?;
        let mut rx_document_batcher = step_batch_10240
            .link(
                rx_compressor.pop().unwrap(),
                docstore_batcher_progress.clone(),
                vec![writter_progress.clone()],
            )
            .await?;
        let mut rx_writter = step_save
            .link(
                rx_document_batcher.pop().unwrap(),
                writter_progress.clone(),
                vec![docstore_completed_progress.clone()],
            )
            .await?;

        let _ = t.send(wiki_xml);

        let mut rx_writter = rx_writter.pop().unwrap();
        loop {
            let _x = rx_writter.recv().await;
        }
    }
}
fn new_progress_bar(multibar: &MultiProgress, limit: u64) -> Arc<ProgressBar> {
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap();

    let pb = multibar.add(ProgressBar::new(limit));
    pb.set_style(sty);
    Arc::new(pb)
}
#[cfg(test)]
mod test {

    use async_openai::{config::OpenAIConfig, Client};
    use indicatif_log_bridge::LogWrapper;

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 32)]
    async fn test() -> Result<(), PipelineError> {
        log::info!("ok");

        let logger =
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .build();

        let multi_progress = MultiProgress::new();

        LogWrapper::new(multi_progress.clone(), logger)
            .try_init()
            .unwrap();

        let pipeline = PipelineProcessor;
        let openai_config = OpenAIConfig::new().with_api_base("http://localhost:9000/v1");
        let open_ai_client: Client<OpenAIConfig> = Client::with_config(openai_config);
        let embedding_client =
            EmbeddingClient::new(open_ai_client, "thenlper/gte-small".to_string());

        let _ = pipeline
            .process(
                &multi_progress,
                PathBuf::from("/home/michael/Desktop/wikisql/enwiki-20240420-pages-articles.xml"),
                PathBuf::from("/home/michael/Desktop/wikisql/wikipedia_docstore_20240420.sqlite"),
                embedding_client,
            )
            .await;

        Ok(())
    }
}
