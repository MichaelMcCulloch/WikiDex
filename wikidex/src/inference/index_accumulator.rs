use std::collections::HashMap;

pub(crate) struct IndexAccumulator {
    dictionary: HashMap<i64, u8>,
    token_buffer: Vec<String>,
}

pub(crate) enum IndexAccumulatorReturn<'a> {
    Nothing,
    NoTransform(&'a str),
    Transform(String),
}

pub(crate) trait IndexAccumulatorTrait {
    fn token(&mut self, token: &str) -> IndexAccumulatorReturn;
}

impl IndexAccumulatorTrait for IndexAccumulator {
    fn token(&mut self, _token: &str) -> IndexAccumulatorReturn {
        todo!()
    }
}
