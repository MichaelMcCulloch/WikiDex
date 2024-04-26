use super::PipelineStep;
use crate::{
    embedding_client::{EmbeddingClient, EmbeddingClientService},
    ingest::pipeline::{
        document::{DocumentHeading, DocumentTextHeadingEmbedding},
        error::{
            EmbeddingError::{self, EmbeddingServiceError as EmbedError},
            PipelineError,
        },
    },
};

use backoff::{future::retry, Error as Backoff, ExponentialBackoff};

use std::sync::Arc;

const EMBED_MAX_STR_LEN_ACCORDING_TO_INFINITY: usize = 122880usize;
pub(crate) struct Embedding {
    client: Arc<EmbeddingClient>,
}
impl Embedding {
    pub(crate) fn new(embedding_client: EmbeddingClient) -> Self {
        Self {
            client: Arc::new(embedding_client),
        }
    }

    pub async fn get_embeddings(
        embedder: &EmbeddingClient,
        queries: &[String],
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        retry(ExponentialBackoff::default(), || async {
            embedder
                .embed_batch(queries.to_vec())
                .await
                .map_err(|e| Backoff::transient(EmbedError(e)))
        })
        .await
    }
}

impl PipelineStep<true> for Embedding {
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
            .map(|d| {
                format!("{d}")
                    .chars()
                    .take(EMBED_MAX_STR_LEN_ACCORDING_TO_INFINITY)
                    .collect()
            })
            .collect::<Vec<_>>();

        let _ = retry(ExponentialBackoff::default(), || async {
            Ok(embedder.up().await?)
        })
        .await;

        let embeddings = Self::get_embeddings(embedder, &queries).await?;

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
                    document_id: document.document_id,
                    article_id: document.article_id,
                },
            )
            .collect::<Vec<_>>();
        Ok(documents)
    }

    fn args(&self) -> Self::ARG {
        self.client.clone()
    }
}
