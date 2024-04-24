use std::{marker::PhantomData, sync::Arc};

use indicatif::ProgressBar;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use crate::ingest::pipeline::error::PipelineError;

use super::PipelineStep;
#[derive(Default)]
pub(crate) struct PipelineSplitter<X: Clone + Sync + Send + 'static> {
    _x: PhantomData<X>,
}

impl<X: Clone + Sync + Send + 'static> PipelineStep for PipelineSplitter<X> {
    type IN = X;

    type ARG = ();

    type OUT = X;

    fn name() -> String {
        todo!()
    }

    async fn transform(_input: Self::IN, _arg: &Self::ARG) -> Vec<Self::OUT> {
        todo!()
    }

    fn args(&self) -> Self::ARG {
        todo!()
    }

    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
        _progress: Arc<ProgressBar>,
        _next_progress: Vec<Arc<ProgressBar>>,
    ) -> Result<Vec<UnboundedReceiver<Self::OUT>>, PipelineError> {
        let (sender1, new_receiver1) = unbounded_channel::<Self::OUT>();
        let (sender2, new_receiver2) = unbounded_channel::<Self::OUT>();

        tokio::spawn(async move {
            while let Some(input) = receiver.recv().await {
                let _ = sender1.send(input.clone());
                let _ = sender2.send(input);
            }
        });
        Ok(vec![new_receiver1, new_receiver2])
    }
}
