use std::{error::Error, path::Path};

#[async_trait::async_trait]
pub(crate) trait Ingest {
    type E: Error;
    async fn ingest_wikipedia(
        self,
        input_xml: &Path,
        output_directory: &Path,
    ) -> Result<usize, Self::E>;
}
