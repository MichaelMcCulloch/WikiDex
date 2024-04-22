mod recursive_text_splitter;
mod wikipedia_dump_reader;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub(crate) use recursive_text_splitter::Splitter;
pub(crate) use wikipedia_dump_reader::WikipediaDumpReader;

pub(crate) trait PipelineStep {
    type IN: Send + Sync;
    type OUT: Send + Sync;
    async fn link(
        &self,
        receiver: UnboundedReceiver<Self::IN>,
        sender: UnboundedSender<Self::OUT>,
    ) -> Result<(), super::error::PipelineError>;
}
