use crate::ingest::{
    pipeline::{
        recursive_character_text_splitter::RecursiveCharacterTextSplitter,
        steps::{Splitter, WikipediaDumpReader},
    },
    wikipedia::WikiMarkupProcessor,
};

async fn whatever() {
    let recursive_splitter = RecursiveCharacterTextSplitter::new(1024, 128, None, true);
    let processor = WikiMarkupProcessor;
    let _reader = WikipediaDumpReader::new(processor, 1000);
    let _splitter = Splitter::new(recursive_splitter);
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use tokio::sync::mpsc::unbounded_channel;

    use crate::ingest::pipeline::{error::PipelineError, steps::PipelineStep};

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 32)]
    async fn test() -> Result<(), PipelineError> {
        log::info!("ok");
        let recursive_splitter = RecursiveCharacterTextSplitter::new(1024, 128, None, true);
        let processor = WikiMarkupProcessor;
        let reader = WikipediaDumpReader::new(processor, 1);
        let splitter = Splitter::new(recursive_splitter);

        let (t, r) = unbounded_channel::<PathBuf>();

        let r = reader.link(r).await?;
        let mut r = splitter.link(r).await?;

        let _ = t.send(PathBuf::from(
            "/home/michael/Documents/WIKIDUMPS/20240401/enwiki-20240401-pages-articles.xml",
        ));

        // while let Ok(Some(document)) = timeout(Duration::from_secs(10), r.recv()).await {
        while let Some(document) = r.recv().await {
            println!("{}", document.document);
            println!("{}", ["="; 160].join(""));
        }
        Ok(())
    }
}
