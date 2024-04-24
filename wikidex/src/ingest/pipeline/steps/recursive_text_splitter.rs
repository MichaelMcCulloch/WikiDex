use std::sync::Arc;

use crate::ingest::pipeline::{
    document::DocumentHeading, error::PipelineError,
    recursive_character_text_splitter::RecursiveCharacterTextSplitter,
};

use super::PipelineStep;
const MINIMUM_PASSAGE_LENGTH_IN_WORDS: usize = 15;
const CHUNK_SIZE: usize = 1024;
const CHUNK_OVERLAP: usize = 128;
pub(crate) struct Splitter {
    splitter: Arc<RecursiveCharacterTextSplitter>,
}

// WARN: You need a lot of memory to use this in conjunction with the wikipedia dump reader; 128GB is not enough for a full dump of wikipedia.
impl Splitter {
    pub(crate) fn new(splitter: RecursiveCharacterTextSplitter) -> Self {
        Self {
            splitter: Arc::new(splitter),
        }
    }
}
impl PipelineStep for Splitter {
    type IN = DocumentHeading;

    type OUT = DocumentHeading;

    type ARG = Arc<RecursiveCharacterTextSplitter>;

    async fn transform(input: Self::IN, arg: &Self::ARG) -> Result<Vec<Self::OUT>, PipelineError> {
        Ok(arg
            .split_text(&input.document)
            .into_iter()
            .filter(|passage| {
                passage.split(' ').collect::<Vec<_>>().len() > MINIMUM_PASSAGE_LENGTH_IN_WORDS
            })
            .map(|document| DocumentHeading {
                document,
                heading: input.heading.clone(),
                article_title: input.article_title.clone(),
                access_date: input.access_date,
                modification_date: input.modification_date,
            })
            .collect::<Vec<_>>())
    }

    fn args(&self) -> Self::ARG {
        self.splitter.clone()
    }
    fn name() -> String {
        String::from("Splitter")
    }
}
