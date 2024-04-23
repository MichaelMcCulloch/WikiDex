use std::sync::Arc;

use crate::ingest::pipeline::document::Document;

use super::PipelineStep;
const MINIMUM_PASSAGE_LENGTH_IN_WORDS: usize = 15;
const CHUNK_SIZE: usize = 1024;
const CHUNK_OVERLAP: usize = 128;

//###HEADING###

pub(crate) struct PatternSplitter {
    pattern: Arc<String>,
}

impl PatternSplitter {
    pub(crate) fn new(pattern: String) -> Self {
        Self {
            pattern: Arc::new(pattern),
        }
    }
}
impl PipelineStep for PatternSplitter {
    type IN = Document;

    type OUT = Document;

    type ARG = Arc<String>;

    async fn transform(input: Self::IN, arg: &Self::ARG) -> Vec<Self::OUT> {
        let Document {
            document,
            article_title,
            access_date,
            modification_date,
        } = input;

        document
            .split(&**arg)
            .filter(|passage| {
                passage.split(' ').collect::<Vec<_>>().len() > MINIMUM_PASSAGE_LENGTH_IN_WORDS
            })
            .map(|document| Document {
                document: document.to_string(),
                article_title: article_title.clone(),
                access_date,
                modification_date,
            })
            .collect::<Vec<_>>()
    }

    fn args(&self) -> Self::ARG {
        self.pattern.clone()
    }
    fn name() -> String {
        String::from("PatternSplitter")
    }
}
