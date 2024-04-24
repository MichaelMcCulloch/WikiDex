use std::sync::Arc;

use super::PipelineStep;
use crate::{
    embedding_client::{EmbeddingClient, EmbeddingClientService},
    ingest::pipeline::document::DocumentHeading,
};

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

    type OUT = Vec<Vec<f32>>;

    fn name() -> String {
        String::from("Embed")
    }

    async fn transform(_input: Self::IN, _arg: &Self::ARG) -> Vec<Self::OUT> {
        let queries = _input
            .into_iter()
            .map(|d| format!("{d}"))
            .collect::<Vec<_>>();
        vec![_arg.embed_batch(queries).await.unwrap()]
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
        let next_progress = next_progress.first().unwrap().clone();

        progress.set_message(Self::name().to_string());
        tokio::spawn(async move {
            while let Some(input) = receiver.recv().await {
                let args = args.clone();
                let sender = sender.clone();
                let progress = progress.clone();
                let next_progress = next_progress.clone();

                let transform = Self::transform(input, &args).await;
                progress.inc(1);

                for t in transform {
                    next_progress.inc_length(1);

                    let _ = sender.send(t);
                }
            }
        });
        Ok(vec![new_receiver])
    }
}
