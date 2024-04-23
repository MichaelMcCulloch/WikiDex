use std::sync::Arc;

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use crate::ingest::pipeline::{
    document::DocumentWithHeading, error::PipelineError,
    recursive_character_text_splitter::RecursiveCharacterTextSplitter,
};

use super::PipelineStep;
const MINIMUM_PASSAGE_LENGTH_IN_WORDS: usize = 15;
const CHUNK_SIZE: usize = 1024;
const CHUNK_OVERLAP: usize = 128;
pub(crate) struct Splitter {
    splitter: Arc<RecursiveCharacterTextSplitter>,
}

impl Splitter {
    pub(crate) fn new(splitter: RecursiveCharacterTextSplitter) -> Self {
        Self {
            splitter: Arc::new(splitter),
        }
    }
}
impl PipelineStep for Splitter {
    type IN = DocumentWithHeading;

    type OUT = DocumentWithHeading;

    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
    ) -> Result<UnboundedReceiver<Self::OUT>, PipelineError> {
        let (t, r) = unbounded_channel::<Self::OUT>();
        let splitter = self.splitter.clone();
        tokio::spawn(async move {
            while let Some(input) = receiver.recv().await {
                let DocumentWithHeading {
                    document,
                    heading,
                    article_title,
                    access_date,
                    modification_date,
                } = input;
                let documents: Vec<DocumentWithHeading> = splitter
                    .split_text(&document)
                    .into_iter()
                    .filter(|passage| {
                        passage.split(' ').collect::<Vec<_>>().len()
                            > MINIMUM_PASSAGE_LENGTH_IN_WORDS
                    })
                    .map(|document| DocumentWithHeading {
                        document,
                        heading: heading.clone(),
                        article_title: article_title.clone(),
                        access_date,
                        modification_date,
                    })
                    .collect::<Vec<_>>();

                for document in documents.into_iter() {
                    let _ = t.send(document);
                }
            }
            Ok::<(), PipelineError>(())
        });
        Ok(r)
    }
}
