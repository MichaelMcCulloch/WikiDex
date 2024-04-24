use std::{path::PathBuf, time::Duration};

use std::sync::{atomic::AtomicUsize, Arc};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::sync::mpsc::unbounded_channel;
use url::Url;

use crate::ingest::pipeline::{error::PipelineError, steps::PipelineStep};
use crate::ingest::pipeline::{
    recursive_character_text_splitter::RecursiveCharacterTextSplitter,
    steps::{Splitter, WikipediaDumpReader},
};

use super::document::CompressedDocument;
use super::steps::{Batcher, SqliteWriter};
use super::{
    steps::{Compressor, WikipediaHeadingSplitter, WikipediaPageParser},
    wikipedia::WikiMarkupProcessor,
};

pub(crate) struct PipelineProcessor;

impl PipelineProcessor {
    pub(crate) async fn process(
        &self,
        multibar: &MultiProgress,
        wiki_xml: PathBuf,
        database_connection: Url,
    ) -> Result<(), PipelineError> {
        let options: SqliteConnectOptions = database_connection.to_string().parse().unwrap();

        let options: SqliteConnectOptions = options.pragma("locking_mode", "on");
        let options: SqliteConnectOptions = options.pragma("journal_mode", "on");

        let pool = SqlitePoolOptions::new()
            .acquire_timeout(Duration::from_secs(10000))
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();

        let reader = WikipediaDumpReader::new(0);
        let parser = WikipediaPageParser::new(WikiMarkupProcessor);
        let wikisplitter = WikipediaHeadingSplitter;
        let splitter = Splitter::new(RecursiveCharacterTextSplitter::new(2048, 0, None, true));
        let compressor = Compressor;
        let batcher = Batcher::<10000, CompressedDocument>::default();

        let writter = SqliteWriter::new(pool).await;

        let (t, r) = unbounded_channel::<PathBuf>();

        let reader_progress = Arc::new(new_progress_bar(multibar, 0));
        let parser_progress = Arc::new(new_progress_bar(multibar, 0));
        let wikisplitter_progress = Arc::new(new_progress_bar(multibar, 0));
        let splitter_progress = Arc::new(new_progress_bar(multibar, 0));
        let compressor_progress = Arc::new(new_progress_bar(multibar, 0));
        let batcher_progress = Arc::new(new_progress_bar(multibar, 0));
        let writter_progress = Arc::new(new_progress_bar(multibar, 0));
        let completed_progress = Arc::new(new_progress_bar(multibar, 0));
        completed_progress.set_message("DONE");
        let r = reader
            .link(r, reader_progress.clone(), parser_progress.clone())
            .await?;
        let r = parser
            .link(r, parser_progress.clone(), wikisplitter_progress.clone())
            .await?;
        let r = wikisplitter
            .link(r, wikisplitter_progress.clone(), splitter_progress.clone())
            .await?;
        let r = splitter
            .link(r, splitter_progress.clone(), compressor_progress.clone())
            .await?;
        let r = compressor
            .link(r, compressor_progress.clone(), batcher_progress.clone())
            .await?;
        let r = batcher
            .link(r, batcher_progress.clone(), writter_progress.clone())
            .await?;
        let mut r = writter
            .link(r, writter_progress.clone(), completed_progress.clone())
            .await?;

        let _ = t.send(wiki_xml);

        let _o = AtomicUsize::new(0);
        // while let Ok(Some(document)) = timeout(Duration::from_secs(10), r.recv()).await {
        while let Some(_document) = r.recv().await {
            // println!("{}", o.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
        }
        Ok(())
    }
}
fn new_progress_bar(multibar: &MultiProgress, limit: u64) -> ProgressBar {
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap();

    let pb = multibar.add(ProgressBar::new(limit));
    pb.set_style(sty);
    pb
}
#[cfg(test)]
mod test {

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

        let options = SqliteConnectOptions::new()
            .create_if_missing(true)
            .filename("/home/michael/Desktop/wikisql/wikipedia_docstore_20240401.sqlite");

        let options: SqliteConnectOptions = options.pragma("locking_mode", "EXCLUSIVE");
        let options: SqliteConnectOptions = options.pragma("journal_mode", "WAL");
        let options: SqliteConnectOptions = options.pragma("synchronous", "normal");
        let options: SqliteConnectOptions = options.pragma("temp_store", "memory");
        let options: SqliteConnectOptions = options.pragma("mmap_size", "30000000000");

        let pool = SqlitePoolOptions::new()
            .acquire_timeout(Duration::from_secs(10000))
            .max_connections(32)
            .connect_with(options)
            .await
            .unwrap();

        let reader = WikipediaDumpReader::new(0);
        let parser = WikipediaPageParser::new(WikiMarkupProcessor);
        let wikisplitter = WikipediaHeadingSplitter;
        let splitter = Splitter::new(RecursiveCharacterTextSplitter::new(2048, 0, None, true));
        let compressor = Compressor;
        let batcher = Batcher::<100000, CompressedDocument>::default();

        let writter = SqliteWriter::new(pool).await;

        let (t, r) = unbounded_channel::<PathBuf>();

        let reader_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        let parser_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        let wikisplitter_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        let splitter_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        let compressor_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        let batcher_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        let writter_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        let completed_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        completed_progress.set_message("DONE");
        let r = reader
            .link(r, reader_progress.clone(), parser_progress.clone())
            .await?;
        let r = parser
            .link(r, parser_progress.clone(), wikisplitter_progress.clone())
            .await?;
        let r = wikisplitter
            .link(r, wikisplitter_progress.clone(), splitter_progress.clone())
            .await?;
        let r = splitter
            .link(r, splitter_progress.clone(), compressor_progress.clone())
            .await?;
        let r = compressor
            .link(r, compressor_progress.clone(), batcher_progress.clone())
            .await?;
        let r = batcher
            .link(r, batcher_progress.clone(), writter_progress.clone())
            .await?;
        let mut r = writter
            .link(r, writter_progress.clone(), completed_progress.clone())
            .await?;

        let _ = t.send(PathBuf::from(
            "/home/michael/Documents/WIKIDUMPS/20240401/enwiki-20240401-pages-articles.xml",
        ));

        let _o = AtomicUsize::new(0);
        // while let Ok(Some(document)) = timeout(Duration::from_secs(10), r.recv()).await {
        while let Some(_document) = r.recv().await {
            // println!("{}", o.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
        }
        Ok(())
    }
}
