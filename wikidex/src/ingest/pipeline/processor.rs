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
        wiki_xml_path: PathBuf,
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

        let step_read_input = WikipediaDumpReader::new(0);
        let step_parse_markup = WikipediaMarkdownParser::new(WikiMarkupProcessor);
        let step_split_on_heading = WikipediaHeadingSplitter::default();
        let step_batch_2048 = Batcher::<2048, DocumentHeading>::new();
        let step_embed = Embedding::new(embedding_client);
        let step_compress = Compressor;
        let step_batch_10240 = Batcher::<10240, DocumentCompressed>::new();
        let step_save = SqliteWriter::new(docstore_pool, index_pool).await?;

        let progres_read_input = new_progress_bar(multi_progress, 0);
        let progres_parse_markup = new_progress_bar(multi_progress, 0);
        let progres_split_on_heading = new_progress_bar(multi_progress, 0);
        let progres_batch_2048 = new_progress_bar(multi_progress, 0);
        let progres_embed = new_progress_bar(multi_progress, 0);
        let progres_compress = new_progress_bar(multi_progress, 0);
        let progres_batch_10240 = new_progress_bar(multi_progress, 0);
        let progres_save = new_progress_bar(multi_progress, 0);
        let progres_docstore = new_progress_bar(multi_progress, 0);

        progres_docstore.set_message("Docstore");

        let (t, rx_pathbuf) = unbounded_channel::<PathBuf>();
        let mut rx_page = step_read_input
            .link(
                rx_pathbuf,
                progres_read_input.clone(),
                vec![progres_parse_markup.clone()],
            )
            .await?;
        let mut rx_document = step_parse_markup
            .link(
                rx_page.pop().unwrap(),
                progres_parse_markup.clone(),
                vec![progres_split_on_heading.clone()],
            )
            .await?;
        let mut rx_doc_heading = step_split_on_heading
            .link(
                rx_document.pop().unwrap(),
                progres_split_on_heading.clone(),
                vec![progres_batch_2048.clone()],
            )
            .await?;
        let mut rx_batch_2048 = step_batch_2048
            .link(
                rx_doc_heading.pop().unwrap(),
                progres_batch_2048.clone(),
                vec![progres_embed.clone()],
            )
            .await?;
        let mut rx_doc_head_embed = step_embed
            .link(
                rx_batch_2048.pop().unwrap(),
                progres_embed,
                vec![progres_compress.clone()],
            )
            .await?;
        let mut rx_doc_compress = step_compress
            .link(
                rx_doc_head_embed.pop().unwrap(),
                progres_compress.clone(),
                vec![progres_batch_10240.clone()],
            )
            .await?;
        let mut rx_batch_10240 = step_batch_10240
            .link(
                rx_doc_compress.pop().unwrap(),
                progres_batch_10240.clone(),
                vec![progres_save.clone()],
            )
            .await?;
        let mut rx_written = step_save
            .link(
                rx_batch_10240.pop().unwrap(),
                progres_save.clone(),
                vec![progres_docstore.clone()],
            )
            .await?;

        let _ = t.send(wiki_xml_path);

        let mut rx_writter = rx_written.pop().unwrap();
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
