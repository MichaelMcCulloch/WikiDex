use std::sync::Arc;

use crate::ingest::pipeline::{document::Document, error::PipelineError};

use super::PipelineStep;
const MINIMUM_PASSAGE_LENGTH_IN_WORDS: usize = 15;

//###HEADING###

pub(crate) struct PatternSplitter {
    pattern: Arc<String>,
}

impl PatternSplitter {
    pub(crate) fn _new(pattern: String) -> Self {
        Self {
            pattern: Arc::new(pattern),
        }
    }
}
impl PipelineStep<true> for PatternSplitter {
    type IN = Document;

    type OUT = Document;

    type ARG = Arc<String>;

    async fn transform(input: Self::IN, arg: &Self::ARG) -> Result<Vec<Self::OUT>, PipelineError> {
        Ok(input
            .document
            .split(&**arg)
            .filter(|passage| {
                passage.split(' ').collect::<Vec<_>>().len() > MINIMUM_PASSAGE_LENGTH_IN_WORDS
            })
            .map(|document| Document {
                document: document.to_string(),
                article_title: input.article_title.clone(),
                access_date: input.access_date,
                modification_date: input.modification_date,
                article_id: input.article_id,
            })
            .collect::<Vec<_>>())
    }

    fn args(&self) -> Self::ARG {
        self.pattern.clone()
    }
    fn name() -> String {
        String::from("PatternSplitter")
    }
}
