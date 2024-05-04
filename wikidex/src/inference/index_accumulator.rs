use std::collections::HashMap;

pub(crate) struct IndexAccumulator {
    dictionary: HashMap<i64, u8>,
    token_buffer: Vec<String>,
    is_accumulating: bool,
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
    fn token(&mut self, token: &str) -> IndexAccumulatorReturn {
        if token.trim().parse::<i64>().is_ok() {
            self.token_buffer.push(token.to_string());
            IndexAccumulatorReturn::Nothing
        } else {
            IndexAccumulatorReturn::Nothing
        }
    }
}
