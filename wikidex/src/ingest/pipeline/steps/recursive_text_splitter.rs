use std::sync::Arc;

use crate::ingest::pipeline::{
    document::DocumentWithHeading,
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

    type ARG = Arc<RecursiveCharacterTextSplitter>;

    async fn transform(input: Self::IN, arg: &Self::ARG) -> Vec<Self::OUT> {
        let DocumentWithHeading {
            document,
            heading,
            article_title,
            access_date,
            modification_date,
        } = input;
        arg.split_text(&document)
            .into_iter()
            .filter(|passage| {
                passage.split(' ').collect::<Vec<_>>().len() > MINIMUM_PASSAGE_LENGTH_IN_WORDS
            })
            .map(|document| DocumentWithHeading {
                document,
                heading: heading.clone(),
                article_title: article_title.clone(),
                access_date,
                modification_date,
            })
            .collect::<Vec<_>>()
    }

    fn args(&self) -> Self::ARG {
        self.splitter.clone()
    }
}
