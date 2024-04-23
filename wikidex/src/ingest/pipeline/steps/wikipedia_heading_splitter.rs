use crate::ingest::pipeline::document::DocumentWithHeading;

use crate::ingest::pipeline::document::Document;
use crate::ingest::pipeline::{HEADING_END, HEADING_START};

use super::PipelineStep;

pub(crate) struct WikipediaHeadingSplitter;

impl PipelineStep for WikipediaHeadingSplitter {
    type IN = Document;

    type OUT = DocumentWithHeading;

    type ARG = ();

    async fn transform(input: Self::IN, _arg: &Self::ARG) -> Vec<Self::OUT> {
        let Document {
            document,
            article_title,
            access_date,
            modification_date,
        } = input;
        document
            .split(HEADING_START)
            .map(|s| {
                let split = s.split(HEADING_END).collect::<Vec<_>>();

                match split.len() {
                    2 => {
                        let heading = format!("{}{}", article_title, split.first().unwrap());
                        let text = split.get(1).unwrap().to_string();
                        (heading, text)
                    }
                    1 => {
                        let text = format!("{}{}", article_title, split.first().unwrap());
                        (String::new(), text)
                    }
                    _ => (String::new(), split.join("")),
                }
            })
            .map(|(heading, document)| DocumentWithHeading {
                document: document.trim().to_string(),
                heading,
                article_title: article_title.clone(),
                access_date,
                modification_date,
            })
            .collect::<Vec<_>>()
    }

    fn args(&self) -> Self::ARG {}
    fn name() -> String {
        String::from("WikipediaHeadingSplitter")
    }
}
