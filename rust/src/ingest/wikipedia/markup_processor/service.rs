use std::error::Error;

pub(crate) trait Process {
    type E: Error;
    fn process(&self, markup: &str) -> Result<String, Self::E>;
}
