use std::{error::Error, path::Path};
pub(crate) trait Ingest {
    type E: Error;
    fn ingest_wikipedia(self, input_xml: &Path, output_directory: &Path) -> Result<usize, Self::E>;
}
