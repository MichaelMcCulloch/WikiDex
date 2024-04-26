use std::sync::Arc;

use crate::ingest::pipeline::error::{BatchingError, LinkError, PipelineError};
use indicatif::ProgressBar;
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver},
        RwLock,
    },
    time::Duration,
};

use super::PipelineStep;

pub(crate) struct Batcher<const N: usize, X: Sync + Send + 'static> {
    batch: Arc<RwLock<Option<Vec<X>>>>,
}

impl<const N: usize, X: Sync + Send + 'static> Batcher<N, X> {
    pub(crate) fn new() -> Self {
        Self {
            batch: Arc::new(RwLock::new(Some(vec![]))),
        }
    }
}

impl<const N: usize, X: Sync + Send + 'static> PipelineStep<false> for Batcher<N, X> {
    type IN = X;
    type ARG = Arc<RwLock<Option<Vec<X>>>>;
    type OUT = Vec<X>;

    fn name() -> String {
        format!("Batch {}", N)
    }

    fn args(&self) -> Self::ARG {
        self.batch.clone()
    }
    async fn transform(
        input: Self::IN,
        batch: &Self::ARG,
    ) -> Result<Vec<Self::OUT>, PipelineError> {
        let mut batch = batch.write().await;

        if let Some(bat) = batch.as_mut() {
            bat.push(input);
            if bat.len() >= N {
                match batch.replace(vec![]) {
                    Some(replace) => Ok(vec![replace]),
                    None => Err(BatchingError::CouldNotObtainBatch)?,
                }
            } else {
                Ok(vec![])
            }
        } else {
            Err(BatchingError::CouldNotObtainBatch)?
        }
    }
    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
        progress: Arc<ProgressBar>,
        next_progress: Vec<Arc<ProgressBar>>,
    ) -> Result<Vec<UnboundedReceiver<Self::OUT>>, PipelineError> {
        let (sender, new_receiver) = unbounded_channel::<Self::OUT>();
        let batch = self.args();
        let next_progress = next_progress
            .first()
            .ok_or(LinkError::NoCurrentProgressBar(Self::name()))?
            .clone();

        progress.set_message(Self::name().to_string());
        tokio::spawn(async move {
            let mut last_flush = tokio::time::Instant::now();
            let flush_timeout = Duration::from_secs(10);

            let pipeline_batch = batch.clone();
            let pipeline_sender = sender.clone();
            let pipeline_progress = progress.clone();
            let pipeline_next_progress = next_progress.clone();

            // Timeout task to handle flushing the batch after inactivity
            tokio::spawn(async move {
                let batch = batch.clone();
                let sender = sender.clone();
                let progress = progress.clone();
                let next_progress = next_progress.clone();
                loop {
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    if tokio::time::Instant::now() - last_flush >= flush_timeout {
                        let mut batch = batch.write().await;

                        if batch.as_ref().is_some() {
                            match batch.replace(vec![]) {
                                Some(replace) => {
                                    if !replace.is_empty() {
                                        progress.inc(1);
                                        next_progress.inc_length(1);

                                        let _ = sender.send(replace);
                                    }
                                }
                                None => continue,
                            };
                        } else {
                            continue;
                        }
                    }
                }
            });
            while let Some(input) = receiver.recv().await {
                last_flush = tokio::time::Instant::now();

                let transform = Self::transform(input, &pipeline_batch)
                    .await
                    .map_err(PipelineError::from);

                match transform {
                    Ok(transform) => {
                        pipeline_progress.inc(1);

                        for t in transform {
                            pipeline_next_progress.inc_length(1);

                            let _ = pipeline_sender.send(t);
                        }
                    }
                    Err(e) => {
                        log::warn!("{} {e}", Self::name())
                    }
                }
            }

            Ok::<(), PipelineError>(())
        });
        Ok(vec![new_receiver])
    }
}
