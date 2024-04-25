use std::{marker::PhantomData, sync::Arc};

use indicatif::ProgressBar;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use crate::ingest::pipeline::error::{LinkError, PipelineError};

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
        format!("Batch {}", N)
    }

    fn args(&self) -> Self::ARG {
        unimplemented!()
    }

    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
        progress: Arc<ProgressBar>,
        next_progress: Vec<Arc<ProgressBar>>,
    ) -> Result<Vec<UnboundedReceiver<Self::OUT>>, PipelineError> {
        let (sender, new_receiver) = unbounded_channel();
        let next_progress = next_progress
            .first()
            .ok_or(LinkError::NoCurrentProgressBar)?
            .clone();

        progress.set_message(Self::name().to_string());

        tokio::spawn(async move {
            let mut batch = Some(vec![]);
            let progress = progress.clone();
            let next_progress = next_progress.clone();
            while let Some(input) = receiver.recv().await {
                let sender = sender.clone();
                let next_progress = next_progress.clone();
                let progress = progress.clone();

                if let Some(v) = batch.as_mut() {
                    v.push(input);
                }
                if let Some(ref vec) = batch {
                    if vec.len() > N {
                        progress.inc(1);
                        next_progress.inc_length(1);
                        match batch.replace(vec![]) {
                            Some(replace) => {
                                let _ = sender.send(replace);
                            }
                            None => {
                                log::error!("Could Not Obtain Batch");
                                continue;
                            }
                        };
                    }
                }
            }
        });
        Ok(vec![new_receiver])
    }

    async fn transform(
        _input: Self::IN,
        _arg: &Self::ARG,
    ) -> Result<Vec<Self::OUT>, PipelineError> {
        todo!()
    }
}
