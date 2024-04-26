use std::sync::Arc;

use super::PipelineStep;
use crate::{
    embedding_client::{EmbeddingClient, EmbeddingClientService},
    ingest::pipeline::{
        document::{DocumentHeading, DocumentTextHeadingEmbedding},
        error::{EmbeddingError, PipelineError},
    },
};

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
}

impl PipelineStep<false> for Embedding {
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
