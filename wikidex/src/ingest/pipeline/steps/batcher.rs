use std::{marker::PhantomData, sync::Arc};

use indicatif::ProgressBar;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use crate::ingest::pipeline::error::PipelineError;

use super::PipelineStep;

#[derive(Default)]
pub(crate) struct Batcher<const N: usize, X: Sync + Send + 'static> {
    _phantom: PhantomData<X>,
}

impl<const N: usize, X: Sync + Send + 'static> PipelineStep for Batcher<N, X> {
    type IN = X;

    type ARG = ();

    type OUT = Vec<X>;

    fn name() -> String {
        String::from("Batcher")
    }

    async fn transform(_: Self::IN, _: &Self::ARG) -> Vec<Self::OUT> {
        unimplemented!()
    }

    fn args(&self) -> Self::ARG {
        unimplemented!()
    }

    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
        progress: Arc<ProgressBar>,
        next_progress: Arc<ProgressBar>,
    ) -> Result<UnboundedReceiver<Self::OUT>, PipelineError> {
        let (sender, new_receiver) = unbounded_channel();

        progress.set_message(Self::name().to_string());

        tokio::spawn(async move {
            let mut batch = Some(vec![]);
            while let Some(input) = receiver.recv().await {
                let sender = sender.clone();
                let next_progress = next_progress.clone();
                let progress = progress.clone();

                if let Some(v) = batch.as_mut() {
                    v.push(input);
                }
                progress.inc(1);
                if let Some(ref vec) = batch {
                    if vec.len() > N {
                        next_progress.inc_length(1);
                        let _ = sender.send(batch.replace(vec![]).unwrap());
                    }
                }
            }
        });
        Ok(new_receiver)
    }
}
