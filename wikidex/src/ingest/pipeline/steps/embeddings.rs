use std::sync::Arc;

use futures::TryFutureExt;

use super::PipelineStep;
use crate::{
    embedding_client::{EmbeddingClient, EmbeddingClientService},
    ingest::pipeline::{
        document::{DocumentHeading, DocumentTextHeadingEmbedding},
        error::{EmbeddingError, LinkError, PipelineError},
    },
};

const EMBED_MAX_STR_LEN: usize = 122880usize;
pub(crate) struct Embedding {
    client: Arc<EmbeddingClient>,
}
impl Embedding {
    pub(crate) fn new(embedding_client: EmbeddingClient) -> Self {
        Self {
            client: Arc::new(embedding_client),
        }
    }
}

impl PipelineStep for Embedding {
    type IN = Vec<DocumentHeading>;

    type ARG = Arc<EmbeddingClient>;

    type OUT = DocumentTextHeadingEmbedding;

    fn name() -> String {
        String::from("Embed")
    }

    async fn transform(
        documents: Self::IN,
        embedder: &Self::ARG,
    ) -> Result<Vec<Self::OUT>, PipelineError> {
        let queries = documents
            .clone()
            .into_iter()
            .map(|d| format!("{d}").chars().take(EMBED_MAX_STR_LEN).collect())
            .collect::<Vec<_>>();
        let embeddings = embedder
            .embed_batch(queries.clone())
            .await
            .map_err(EmbeddingError::EmbeddingServiceError)?;
        let documents = documents
            .into_iter()
            .zip(queries)
            .zip(embeddings)
            .map(
                |((document, text), embedding)| DocumentTextHeadingEmbedding {
                    text,
                    article_title: document.article_title,
                    access_date: document.access_date,
                    modification_date: document.modification_date,
                    embedding,
                    heading: document.heading,
                },
            )
            .collect::<Vec<_>>();
        Ok(documents)
    }

    fn args(&self) -> Self::ARG {
        self.client.clone()
    }

    async fn link(
        &self,
        mut receiver: tokio::sync::mpsc::UnboundedReceiver<Self::IN>,
        progress: Arc<indicatif::ProgressBar>,
        next_progress: Vec<Arc<indicatif::ProgressBar>>,
    ) -> Result<
        Vec<tokio::sync::mpsc::UnboundedReceiver<Self::OUT>>,
        crate::ingest::pipeline::error::PipelineError,
    > {
        let (sender, new_receiver) = tokio::sync::mpsc::unbounded_channel::<Self::OUT>();
        let args = Arc::new(self.args());
        let next_progress = next_progress
            .first()
            .ok_or(LinkError::NoCurrentProgressBar)?
            .clone();

        progress.set_message(Self::name().to_string());
        tokio::spawn(async move {
            let progress = progress.clone();
            let next_progress = next_progress.clone();
            while let Some(input) = receiver.recv().await {
                let args = args.clone();
                let sender = sender.clone();
                let progress = progress.clone();
                let next_progress = next_progress.clone();

                let transform = Self::transform(input, &args).await;
                match transform {
                    Ok(transform) => {
                        progress.inc(1);

                        for t in transform {
                            next_progress.inc_length(1);

                            let _ = sender.send(t);
                        }
                    }
                    Err(e) => {
                        log::error!("{e}");
                        continue;
                    }
                }
            }

            Ok::<(), PipelineError>(())
        });
        Ok(vec![new_receiver])
    }
}
