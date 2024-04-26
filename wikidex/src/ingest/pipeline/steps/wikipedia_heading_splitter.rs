use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use crate::ingest::pipeline::document::DocumentHeading;

use super::PipelineStep;
use crate::ingest::pipeline::document::Document;
use crate::ingest::pipeline::error::{PipelineError, WikipediaHeadingSplitterError};
use crate::ingest::pipeline::{HEADING_END, HEADING_START};
#[derive(Default)]
pub(crate) struct WikipediaHeadingSplitter {
    document_id: Arc<AtomicI64>,
}

impl PipelineStep for WikipediaHeadingSplitter {
    type IN = Document;

    type OUT = DocumentHeading;

    type ARG = Arc<AtomicI64>;

    async fn transform(
        input: Self::IN,
        counter: &Self::ARG,
    ) -> Result<Vec<Self::OUT>, PipelineError> {
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
                article_id: input.article_id,
                document_id: counter.fetch_add(1, Ordering::Relaxed),
            }]);
        }
        if starts.len() != ends.len() {
            return Err(WikipediaHeadingSplitterError::HeadingMismatch(
                input.article_title,
            ))?;
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
                        if text.len() > 5 {
                            Some((heading, text))
                        } else {
                            None
                        }
                    }
                    1 => {
                        if s.len() > 5 {
                            Some((input.article_title.to_string(), s.to_string()))
                        } else {
                            None
                        }
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
                document_id: counter.fetch_add(1, Ordering::Relaxed),
                article_id: input.article_id,
            })
            .collect::<Vec<_>>())
    }

    fn args(&self) -> Self::ARG {
        self.document_id.clone()
    }
    fn name() -> String {
        String::from("Wikipedia Heading Splitter")
    }
}
