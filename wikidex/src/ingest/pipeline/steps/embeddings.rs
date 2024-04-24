use std::sync::Arc;

use crate::{
    embedding_client::EmbeddingClient,
    ingest::pipeline::document::{DocumentHeading, DocumentHeadingEmbedding},
};

use super::PipelineStep;

pub(crate) struct Embedding<const N: usize> {
    client: Arc<EmbeddingClient>,
}

impl<const N: usize> PipelineStep for Embedding<N> {
    type IN = [DocumentHeading; N];

    type ARG = Arc<EmbeddingClient>;

    type OUT = [DocumentHeadingEmbedding; N];

    fn name() -> String {
        String::from("Embed")
    }

    async fn transform(_input: Self::IN, _arg: &Self::ARG) -> Vec<Self::OUT> {
        todo!()
    }

    fn args(&self) -> Self::ARG {
        self.client.clone()
    }
}
