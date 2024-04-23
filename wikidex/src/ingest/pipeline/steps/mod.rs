mod gzip_compressor;
mod pattern_text_splitter;
mod recursive_text_splitter;
mod sqlite_writter;
mod wikipedia_dump_reader;
mod wikipedia_heading_splitter;
mod wikipedia_page_parser;
use std::sync::{atomic::AtomicUsize, Arc};

use indicatif::ProgressBar;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

pub(crate) use gzip_compressor::Compressor;
pub(crate) use pattern_text_splitter::PatternSplitter;
pub(crate) use recursive_text_splitter::Splitter;
pub(crate) use sqlite_writter::SqliteWriter;
pub(crate) use wikipedia_dump_reader::WikipediaDumpReader;
pub(crate) use wikipedia_heading_splitter::WikipediaHeadingSplitter;
pub(crate) use wikipedia_page_parser::WikipediaPageParser;

use super::error::PipelineError;

pub(crate) trait PipelineStep {
    type IN: Send + Sync + 'static;
    type ARG: Send + Sync + 'static;
    type OUT: Send + Sync + 'static;

    fn name() -> String;

    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
        progress: Arc<ProgressBar>,
        next_progress: Arc<ProgressBar>,
    ) -> Result<UnboundedReceiver<Self::OUT>, PipelineError> {
        let (sender, new_receiver) = unbounded_channel::<Self::OUT>();
        let args = Arc::new(self.args());

        let o = Arc::new(AtomicUsize::new(0));
        progress.set_message(Self::name().to_string());
        tokio::spawn(async move {
            while let Some(input) = receiver.recv().await {
                let args = args.clone();
                let sender = sender.clone();
                let _o = o.clone();
                let next_progress = next_progress.clone();
                let progress = progress.clone();
                tokio::spawn(async move {
                    let transform = Self::transform(input, &args).await;
                    progress.inc(1);
                    next_progress.inc_length(transform.len() as u64);

                    for t in transform {
                        let _ = sender.send(t);
                    }
                });
            }
        });
        Ok(new_receiver)
    }

    fn transform(
        input: Self::IN,
        arg: &Self::ARG,
    ) -> impl std::future::Future<Output = Vec<Self::OUT>> + std::marker::Send;
    fn args(&self) -> Self::ARG;
}
