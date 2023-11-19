use std::error::Error;

use crate::ingest::wikipedia::helper::wiki::UnlabledDocument;

pub(crate) trait Process {
    type E: Error;
    fn process(&self, markup: &str) -> Result<UnlabledDocument, Self::E>;
}
