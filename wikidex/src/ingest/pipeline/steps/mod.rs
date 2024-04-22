mod recursive_text_splitter;
mod wikipedia_dump_reader;
use tokio::sync::mpsc::{UnboundedReceiver};

pub(crate) use recursive_text_splitter::Splitter;
pub(crate) use wikipedia_dump_reader::WikipediaDumpReader;

use super::error::PipelineError;

pub(crate) trait PipelineStep {
    type IN: Send + Sync;
    type OUT: Send + Sync;
    async fn link(
        &self,
        receiver: UnboundedReceiver<Self::IN>,
    ) -> Result<UnboundedReceiver<Self::OUT>, PipelineError>;
}
