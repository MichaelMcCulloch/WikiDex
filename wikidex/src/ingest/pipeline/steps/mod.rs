mod batcher;
mod embeddings;
mod gzip_compressor;
mod junction;
mod pattern_text_splitter;
mod recursive_text_splitter;
#[cfg(feature = "sqlite")]
mod sqlite_writer;
mod wikipedia_dump_reader;
mod wikipedia_heading_splitter;
mod wikipedia_page_parser;

use indicatif::ProgressBar;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

pub(crate) use batcher::Batcher;
pub(crate) use embeddings::Embedding;
pub(crate) use gzip_compressor::Compressor;

#[cfg(feature = "sqlite")]
pub(crate) use sqlite_writer::SqliteWriter;
pub(crate) use wikipedia_dump_reader::WikipediaDumpReader;
pub(crate) use wikipedia_heading_splitter::WikipediaHeadingSplitter;
pub(crate) use wikipedia_page_parser::WikipediaMarkdownParser;

use super::error::{LinkError, PipelineError};

pub(crate) trait PipelineStep<const ASYNC: bool> {
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
        let next_progress = next_progress
            .first()
            .ok_or(LinkError::NoCurrentProgressBar(Self::name()))?
            .clone();

        progress.set_message(Self::name().to_string());
        tokio::spawn(async move {
            while let Some(input) = receiver.recv().await {
                let args = args.clone();
                let sender = sender.clone();
                let progress = progress.clone();
                let next_progress = next_progress.clone();
                if ASYNC {
                    tokio::spawn(async move {
                        let transform = Self::transform(input, &args)
                            .await
                            .map_err(PipelineError::from);

                        match transform {
                            Ok(transform) => {
                                progress.inc(1);

                                for t in transform {
                                    next_progress.inc_length(1);

                                    let _ = sender.send(t);
                                }
                            }
                            Err(e) => {
                                log::warn!("{} {e}", Self::name())
                            }
                        }

                        Ok::<(), PipelineError>(())
                    });
                } else {
                    let transform = Self::transform(input, &args)
                        .await
                        .map_err(PipelineError::from);

                    match transform {
                        Ok(transform) => {
                            progress.inc(1);

                            for t in transform {
                                next_progress.inc_length(1);

                                let _ = sender.send(t);
                            }
                        }
                        Err(e) => {
                            log::warn!("{} {e}", Self::name())
                        }
                    }
                }
            }

            Ok::<(), PipelineError>(())
        });
        Ok(vec![new_receiver])
    }

    fn transform(
        input: Self::IN,
        arg: &Self::ARG,
    ) -> impl std::future::Future<Output = Result<Vec<Self::OUT>, PipelineError>> + std::marker::Send;
    fn args(&self) -> Self::ARG;
}
