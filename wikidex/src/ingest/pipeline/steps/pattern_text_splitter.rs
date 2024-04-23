use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use crate::ingest::pipeline::{document::Document, error::PipelineError};

use super::PipelineStep;
const MINIMUM_PASSAGE_LENGTH_IN_WORDS: usize = 15;
const CHUNK_SIZE: usize = 1024;
const CHUNK_OVERLAP: usize = 128;

//###HEADING###

pub(crate) struct PatternSplitter {
    pattern: String,
}

impl PatternSplitter {
    pub(crate) fn new(pattern: String) -> Self {
        Self { pattern }
    }
}
impl PipelineStep for PatternSplitter {
    type IN = Document;

    type OUT = Document;

    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
    ) -> Result<UnboundedReceiver<Self::OUT>, PipelineError> {
        let (t, r) = unbounded_channel::<Self::OUT>();
        let pattern = self.pattern.clone();
        tokio::spawn(async move {
            while let Some(input) = receiver.recv().await {
                let Document {
                    document,
                    article_title,
                    access_date,
                    modification_date,
                } = input;

                let documents = document
                    .split(&pattern)
                    .filter(|passage| {
                        passage.split(' ').collect::<Vec<_>>().len()
                            > MINIMUM_PASSAGE_LENGTH_IN_WORDS
                    })
                    .map(|document| Document {
                        document: document.to_string(),
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
