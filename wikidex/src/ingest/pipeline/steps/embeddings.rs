use std::sync::Arc;

use crate::{
    embedding_client::EmbeddingClient,
    ingest::pipeline::document::{DocumentHeading},
};

use super::PipelineStep;

pub(crate) struct Embedding<const N: usize> {
    client: Arc<EmbeddingClient>,
}
impl<const N: usize> Embedding<N> {
    pub(crate) fn new(embedding_client: EmbeddingClient) -> Self {
        Self {
            client: Arc::new(embedding_client),
        }
    }
}

impl<const N: usize> PipelineStep for Embedding<N> {
    type IN = Vec<DocumentHeading>;

    type ARG = Arc<EmbeddingClient>;

    type OUT = Vec<DocumentHeading>;

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
