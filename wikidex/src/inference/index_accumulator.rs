pub(crate) struct IndexAccumulator {
    dictionary: Vec<i64>,
    token_buffer: Vec<String>,
    is_accumulating: bool,
}

#[derive(Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod test {
    use crate::inference::index_accumulator::IndexAccumulatorReturn;

    use super::{IndexAccumulator, IndexAccumulatorTrait};

    #[test]
    fn test() {
        let mut accumulator = IndexAccumulator::new(vec![1234, 4321]);

        let token1 = accumulator.token("This");
        let token2 = accumulator.token(" is");
        let token3 = accumulator.token(" a");
        let token4 = accumulator.token(" test");

        assert_eq!(
            vec![
                IndexAccumulatorReturn::NoOp("This"),
                IndexAccumulatorReturn::NoOp(" is"),
                IndexAccumulatorReturn::NoOp(" a"),
                IndexAccumulatorReturn::NoOp(" test")
            ],
            vec![token1, token2, token3, token4]
        )
    } 
    
    #[test]
    fn test2() {
        let mut accumulator = IndexAccumulator::new(vec![1234, 4321]);

        let token1 = accumulator.token("1");
        let token2 = accumulator.token(" 2");
        let token3 = accumulator.token(" 3");
        let token4 = accumulator.token(" 4");

        assert_eq!(
            vec![
                IndexAccumulatorReturn::NoOp("This"),
                IndexAccumulatorReturn::NoOp(" is"),
                IndexAccumulatorReturn::NoOp(" a"),
                IndexAccumulatorReturn::NoOp(" test")
            ],
            vec![token1, token2, token3, token4]
        )
    }
}
