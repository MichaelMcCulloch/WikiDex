use sqlx::{migrate::MigrateDatabase, SqlitePool};

use crate::ingest::pipeline::{
    recursive_character_text_splitter::RecursiveCharacterTextSplitter,
    steps::{Splitter, WikipediaDumpReader},
};

use super::{
    steps::{
        Compressor, PatternSplitter, SqliteWriter, WikipediaHeadingSplitter, WikipediaPageParser,
    },
    wikipedia::WikiMarkupProcessor,
};

async fn whatever() {
    let recursive_splitter = RecursiveCharacterTextSplitter::new(1024, 128, None, true);
    let _splitter = PatternSplitter::new("###HEADING###".to_string());
    let _wikisplit = WikipediaHeadingSplitter;
    let _processor = WikiMarkupProcessor;
    let _reader = WikipediaDumpReader::new(1000);
    let _reader = WikipediaPageParser::new(WikiMarkupProcessor);
    let _splitter = Splitter::new(recursive_splitter);
    let _compressor = Compressor;

    let path = "string";

    if !sqlx::Sqlite::database_exists(path).await.unwrap() {
        sqlx::Sqlite::create_database(path).await.unwrap();
    }
    let x = SqlitePool::connect(path).await.unwrap();
    let _writter = SqliteWriter::new(x);
}

#[cfg(test)]
mod test {

    use std::{
        path::PathBuf,
        sync::{atomic::AtomicUsize, Arc},
        time::Duration,
    };

    use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
    use indicatif_log_bridge::LogWrapper;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use tokio::sync::mpsc::unbounded_channel;

    use crate::ingest::pipeline::{error::PipelineError, steps::PipelineStep};

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
        let path = "sqlite:///tmp/wikipedia_docstore.sqlite";
        if !sqlx::Sqlite::database_exists(path).await.unwrap() {
            sqlx::Sqlite::create_database(path).await.unwrap();
        }

        let options = SqliteConnectOptions::new()
            .create_if_missing(true)
            .filename("/tmp/wikipedia_docstore.sqlite");

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
        let writter = SqliteWriter::new(pool).await;

        let (t, r) = unbounded_channel::<PathBuf>();

        let reader_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        let parser_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        let wikisplitter_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        let splitter_progress = Arc::new(new_progress_bar(&multi_progress, 0));
        let compressor_progress = Arc::new(new_progress_bar(&multi_progress, 0));
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
        let r: tokio::sync::mpsc::UnboundedReceiver<
            crate::ingest::pipeline::document::DocumentWithHeading,
        > = splitter
            .link(r, splitter_progress.clone(), compressor_progress.clone())
            .await?;
        let r = compressor
            .link(r, compressor_progress.clone(), writter_progress.clone())
            .await?;
        let mut r = writter
            .link(r, writter_progress.clone(), completed_progress.clone())
            .await?;

        let _ = t.send(PathBuf::from(
            "/home/michael/Desktop/enwiki-20240401-pages-articles.xml",
        ));

        let _o = AtomicUsize::new(0);
        // while let Ok(Some(document)) = timeout(Duration::from_secs(10), r.recv()).await {
        while let Some(_document) = r.recv().await {
            // println!("{}", o.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
        }
        Ok(())
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
}
