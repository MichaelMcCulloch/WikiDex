use std::sync::Arc;

use tokio::sync::RwLock;

use crate::ingest::pipeline::error::{BatchingError, PipelineError};

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
}
