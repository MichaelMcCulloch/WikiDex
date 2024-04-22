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

    use crate::ingest::pipeline::{document::Document, steps::PipelineStep};

    use super::*;

    #[actix_rt::test]
    async fn testew() {
        println!("fads");
    }
    #[actix_rt::test]
    async fn test() {
        log::info!("ok");
        let recursive_splitter = RecursiveCharacterTextSplitter::new(1024, 128, None, true);
        let processor = WikiMarkupProcessor;
        let reader = WikipediaDumpReader::new(processor, 1000);
        let splitter = Splitter::new(recursive_splitter);

        let (t, r) = unbounded_channel::<PathBuf>();
        let (tt, rr) = unbounded_channel::<Document>();
        let (ttt, mut rrr) = unbounded_channel::<Document>();

        let _x = reader.link(r, tt).await;
        let _y = splitter.link(rr, ttt).await;

        let _ = t.send(PathBuf::from(
            "/home/michael/Documents/WIKIDUMPS/20240401/enwiki-20240401-pages-articles.xml",
        ));

        while let Some(document) = rrr.recv().await {
            println!("{}", document.article_title);
        }
    }
}
