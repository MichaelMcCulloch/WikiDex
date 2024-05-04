pub(crate) struct IndexAccumulator {
    dictionary: Vec<i64>,
    token_buffer: Vec<String>,
    is_accumulating: bool,
}

pub(crate) enum IndexAccumulatorReturn<'a> {
    Nothing,
    NoTransform(&'a str),
    Transform(String),
}

pub(crate) trait IndexAccumulatorTrait {
    fn token<'a>(&mut self, token: &'a str) -> IndexAccumulatorReturn<'a>;
}

impl IndexAccumulator {
    pub(crate) fn new(dictionary: Vec<i64>) -> Self {
        Self {
            dictionary,
            token_buffer: vec![],
            is_accumulating: false,
        }
    }
}

impl IndexAccumulatorTrait for IndexAccumulator {
    fn token<'a>(&mut self, token: &'a str) -> IndexAccumulatorReturn<'a> {
        if token.trim().parse::<i64>().is_ok() {
            self.token_buffer.push(token.to_string());
            IndexAccumulatorReturn::Nothing
        } else if self.is_accumulating {
            IndexAccumulatorReturn::Transform(self.token_buffer.join(""))
        } else {
            IndexAccumulatorReturn::NoTransform(token)
        }
    }
}
