use crate::ingest::pipeline::document::DocumentHeading;

use super::PipelineStep;
use crate::ingest::pipeline::document::Document;
use crate::ingest::pipeline::error::PipelineError;
use crate::ingest::pipeline::{HEADING_END, HEADING_START};

pub(crate) struct WikipediaHeadingSplitter;

impl PipelineStep for WikipediaHeadingSplitter {
    type IN = Document;

    type OUT = DocumentHeading;

    type ARG = ();

    async fn transform(input: Self::IN, _arg: &Self::ARG) -> Result<Vec<Self::OUT>, PipelineError> {
        let starts = input
            .document
            .match_indices(HEADING_START)
            .collect::<Vec<_>>();
        let ends = input
            .document
            .match_indices(HEADING_END)
            .collect::<Vec<_>>();

        if starts.is_empty() || ends.is_empty() {
            return Ok(vec![DocumentHeading {
                document: input.document.trim().to_string(),
                heading: input.article_title.to_string(),
                article_title: input.article_title.clone(),
                access_date: input.access_date,
                modification_date: input.modification_date,
            }]);
        }
        if starts.len() != ends.len() {
            return Ok(vec![]);
        }

        Ok(input
            .document
            .split(HEADING_START)
            .filter_map(|s| {
                let split = s.split(HEADING_END).collect::<Vec<_>>();
                match split.len() {
                    2 => {
                        let heading = format!("{}{}", input.article_title, split.first()?);
                        let text = split.get(1)?.to_string();
                        Some((heading, text))
                    }
                    1 => {
                        let text = format!("{}{}", input.article_title, split.first()?);
                        Some((String::new(), text))
                    }
                    _ => None,
                }
            })
            .map(|(heading, document)| DocumentHeading {
                document: document.trim().to_string(),
                heading,
                article_title: input.article_title.clone(),
                access_date: input.access_date,
                modification_date: input.modification_date,
            })
            .collect::<Vec<_>>())
    }

    fn args(&self) -> Self::ARG {}
    fn name() -> String {
        String::from("Wikipedia Heading Splitter")
    }
}
