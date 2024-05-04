use std::collections::HashMap;

pub(crate) struct IndexAccumulator {
    dictionary: HashMap<i64, u8>,
    token_buffer: Vec<String>,
}
