pub(crate) struct IndexAccumulator {
    dictionary: Vec<i64>,
    token_buffer: Vec<String>,
    is_accumulating: bool,
}

pub(crate) enum IndexAccumulatorReturn<'a> {
    Nothing,
    NoOp(&'a str),
    Transform(String, usize),
    NoTransform(String),
}

pub(crate) trait IndexAccumulatorTrait {
    fn token<'a>(&mut self, token: &'a str) -> IndexAccumulatorReturn<'a>;
    fn flush(&mut self) -> Option<String>;
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
            self.is_accumulating = true;
            IndexAccumulatorReturn::Nothing
        } else if self.is_accumulating {
            let index_string = self.token_buffer.join("");
            let result = if let Ok(index) = index_string.trim().parse::<i64>() {
                if let Some(position) = self.dictionary.iter().position(|element| element == &index)
                {
                    IndexAccumulatorReturn::Transform(
                        index_string
                            .replace(index.to_string().as_str(), position.to_string().as_str()),
                        position,
                    )
                } else {
                    IndexAccumulatorReturn::NoTransform(index_string.to_string())
                }
            } else {
                self.token_buffer.clear();
                IndexAccumulatorReturn::NoTransform(index_string)
            };

            self.token_buffer.clear();

            self.is_accumulating = false;
            result
        } else {
            IndexAccumulatorReturn::NoOp(token)
        }
    }

    fn flush(&mut self) -> Option<String> {
        let string = self.token_buffer.join("");

        self.token_buffer.clear();
        if string.is_empty() {
            None
        } else {
            Some(string)
        }
    }
}
