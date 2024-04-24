mod batcher;
mod embeddings;
mod gzip_compressor;
mod pattern_text_splitter;
mod pipeline_splitter;
mod recursive_text_splitter;
#[cfg(feature = "sqlite")]
mod sqlite_writter;
mod wikipedia_dump_reader;
mod wikipedia_heading_splitter;
mod wikipedia_page_parser;

use indicatif::ProgressBar;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

pub(crate) use gzip_compressor::Compressor;
pub(crate) use recursive_text_splitter::Splitter;

pub(crate) use batcher::Batcher;
pub(crate) use embeddings::Embedding;
pub(crate) use pipeline_splitter::PipelineSplitter;
#[cfg(feature = "sqlite")]
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
        next_progress: Vec<Arc<ProgressBar>>,
    ) -> Result<Vec<UnboundedReceiver<Self::OUT>>, PipelineError> {
        let (sender, new_receiver) = unbounded_channel::<Self::OUT>();
        let args = Arc::new(self.args());
        let next_progress = next_progress.first().unwrap().clone();

        progress.set_message(Self::name().to_string());
        tokio::spawn(async move {
            while let Some(input) = receiver.recv().await {
                let args = args.clone();
                let sender = sender.clone();
                let progress = progress.clone();
                let next_progress = next_progress.clone();
                tokio::spawn(async move {
                    let transform = Self::transform(input, &args).await;
                    progress.inc(1);

                    for t in transform {
                        next_progress.inc_length(1);

                        let _ = sender.send(t);
                    }
                });
            }
        });
        Ok(vec![new_receiver])
    }

    fn transform(
        input: Self::IN,
        arg: &Self::ARG,
    ) -> impl std::future::Future<Output = Vec<Self::OUT>> + std::marker::Send;
    fn args(&self) -> Self::ARG;
}
