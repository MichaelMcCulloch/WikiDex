use std::sync::Arc;

use actix_web::rt;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::ingest::pipeline::{
    document::Document, error::PipelineError,
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
    type IN = Document;

    type OUT = Document;

    async fn link(
        &self,
        mut receiver: UnboundedReceiver<Self::IN>,
        sender: UnboundedSender<Self::OUT>,
    ) -> Result<(), PipelineError> {
        let splitter = self.splitter.clone();
        rt::spawn(async move {
            while let Some(input) = receiver.recv().await {
                let Document {
                    document,
                    article_title,
                    access_date,
                    modification_date,
                } = input;
                let documents: Vec<Document> = splitter
                    .split_text(&document)
                    .into_iter()
                    .filter(|passage| {
                        passage.split(' ').collect::<Vec<_>>().len()
                            > MINIMUM_PASSAGE_LENGTH_IN_WORDS
                    })
                    .map(|document| Document {
                        document,
                        article_title: article_title.clone(),
                        access_date,
                        modification_date,
                    })
                    .collect::<Vec<_>>();

                for document in documents.into_iter() {
                    let _ = sender.send(document);
                }
            }
            Ok::<(), PipelineError>(())
        });
        Ok(())
    }
}
