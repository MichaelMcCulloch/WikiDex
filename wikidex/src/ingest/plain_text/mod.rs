mod error;
use self::error::PlainTextProcessingError;

use super::service::Process;

#[derive(Clone)]
pub(crate) struct PlainTextProcessor;
impl PlainTextProcessor {
    pub(crate) fn new() -> Self {
        Self
    }
}
impl Process for PlainTextProcessor {
    type E = PlainTextProcessingError;
    fn process(&self, _text: &str) -> Result<String, Self::E> {
        todo!()
    }
}
