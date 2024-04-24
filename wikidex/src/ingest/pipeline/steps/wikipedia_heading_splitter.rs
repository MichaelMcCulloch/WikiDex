use crate::ingest::pipeline::document::DocumentHeading;

use crate::ingest::pipeline::document::Document;
use crate::ingest::pipeline::error::PipelineError;
use crate::ingest::pipeline::{HEADING_END, HEADING_START};

use super::PipelineStep;

pub(crate) struct WikipediaHeadingSplitter;

impl PipelineStep for WikipediaHeadingSplitter {
    type IN = Document;

    type OUT = DocumentHeading;

    type ARG = ();

    async fn transform(input: Self::IN, _arg: &Self::ARG) -> Result<Vec<Self::OUT>, PipelineError> {
        Ok(input
            .document
            .split(HEADING_START)
            .filter_map(|s| {
                let split = s.split(HEADING_END).collect::<Vec<_>>();

                let doc = match split.len() {
                    2 => {
                        let heading = format!("{}{}", input.article_title, split.first()?);
                        let text = split.get(1)?.to_string();
                        (heading, text)
                    }
                    1 => {
                        let text = format!("{}{}", input.article_title, split.first()?);
                        (String::new(), text)
                    }
                    _ => (String::new(), split.join("")),
                };

                if doc.1.trim().is_empty() {
                    None
                } else {
                    Some(doc)
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
        String::from("WikipediaHeadingSplitter")
    }
}
