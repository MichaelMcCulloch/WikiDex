use super::Engine;
use chrono::NaiveDateTime;
use indicatif::ProgressBar;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::{error::Error, path::Path};

pub(crate) trait Ingest {
    type E: Error;
    fn ingest_wikipedia<P: AsRef<Path>>(
        self,
        input_xml: &P,
        output_directory: &P,
    ) -> Result<usize, Self::E>;
}
