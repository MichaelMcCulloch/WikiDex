pub(crate) struct IndexAccumulator {
    dictionary: Vec<i64>,
    token_buffer: Vec<String>,
    is_accumulating: bool,
    formatter: Box<dyn Fn(usize) -> String>,
}

pub(crate) enum IndexAccumulatorReturn<'a> {
    Nothing,
    NoOp(&'a str),
    Transform(String),
    NoTransform(String),
}

pub(crate) trait IndexAccumulatorTrait {
    fn token<'a>(&mut self, token: &'a str) -> IndexAccumulatorReturn<'a>;
    fn flush(self) -> String;
}

impl IndexAccumulator {
    pub(crate) fn new(dictionary: Vec<i64>, formatter: Box<dyn Fn(usize) -> String>) -> Self {
        Self {
            dictionary,
            token_buffer: vec![],
            is_accumulating: false,
            formatter,
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
            let result = if let Ok(index_string) = index_string.trim().parse::<i64>() {
                if let Some(position) = self
                    .dictionary
                    .iter()
                    .position(|element| element == &index_string)
                {
                    let string = (self.formatter)(position);
                    IndexAccumulatorReturn::Transform(string)
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

    fn flush(self) -> String {
        self.token_buffer.join("")
    }
}
