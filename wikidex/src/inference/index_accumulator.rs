use std::collections::HashMap;

pub(crate) struct IndexAccumulator {
    dictionary: HashMap<i64, u8>,
    token_buffer: Vec<String>,
}

pub(crate) trait IndexAccumulatorTrait {
    fn token(&mut self, token: &str) -> Option<&str>;
}
