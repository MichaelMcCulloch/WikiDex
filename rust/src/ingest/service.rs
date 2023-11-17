use std::{error::Error, path::Path};

pub(crate) trait Ingest {
    type E: Error;
    fn ingest_wikipedia<P: AsRef<Path>>(
        self,
        input_xml: &P,
        output_directory: &P,
    ) -> Result<usize, Self::E>;
}
