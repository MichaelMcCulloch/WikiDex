use super::{Ingest, IngestError};
use crate::{embed::Embedder, llm::OpenAiService};
use std::path::Path;

pub(crate) struct Engine {
    embed: Embedder,
    llm: OpenAiService,
}

impl Engine {
    pub(crate) fn new(embed: Embedder, llm: OpenAiService) -> Self {
        Self { embed, llm }
    }
}

impl Ingest for Engine {
    type E = IngestError;

    fn ingest<P: AsRef<Path>>(self, input_xml: &P, output_directory: &P) -> Result<usize, Self::E> {
        match (
            input_xml.as_ref().exists(),
            output_directory.as_ref().exists(),
        ) {
            (true, true) => Ok(1),
            (true, false) => Err(IngestError::OutputDirectoryNotFound(
                output_directory.as_ref().to_path_buf(),
            )),
            (false, _) => Err(IngestError::XMLNotFound(input_xml.as_ref().to_path_buf())),
        }
    }
}
