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
    use crate::inference::index_accumulator::IndexAccumulatorReturn as I;

    use super::{IndexAccumulator, IndexAccumulatorTrait};

    #[test]
    fn test_plain_text() {
        let mut a = IndexAccumulator::new(vec![1234, 4321]);

        assert_eq!(I::NoOp("This"), a.token("This"));
        assert_eq!(I::NoOp(" is"), a.token(" is"));
        assert_eq!(I::NoOp(" a"), a.token(" a"));
        assert_eq!(I::NoOp(" test"), a.token(" test"));
        assert_eq!(None, a.flush());
    }

    #[test]
    fn test_number_is_absent() {
        let mut a = IndexAccumulator::new(vec![1234, 4321]);

        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("4"));
        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(Some("2341".to_string()), a.flush());
    }

    #[test]
    fn test_number_is_present() {
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("4"));
        assert_eq!(Some("0".to_string()), a.flush());
    }
    #[test]
    fn test_two_numbers_are_absent() {
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::NoTransform("123 ".to_string()), a.token(" "));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(Some("321".to_string()), a.flush());
    }
    #[test]
    fn test_two_numbers_are_present_and_same() {
        let mut a = IndexAccumulator::new(vec![123]);

        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Transform("0 ".to_string(), 0), a.token(" "));
        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(Some("1".to_string()), a.flush());
    }
}
