use regex::Regex;

const NON_DIGITS_FOLLOWED_BY_DIGITS: &str = r#"^(\D*)(\d+)$"#;

pub(crate) struct IndexAccumulator {
    dictionary: Vec<i64>,
    token_buffer: Vec<String>,
    modifier: usize,
    is_accumulating: bool,
    non_digits_followed_by_digits: Regex,
    formatter: Box<dyn Fn(usize, usize) -> String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenValue<'a> {
    Nothing,
    NoOp(&'a str),
    Transform(String, usize),
    NoTransform(String),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TokenValues<'a> {
    Nothing,
    Unit(TokenValue<'a>),
    Twofer(TokenValue<'a>, TokenValue<'a>),
}

impl<'a> From<TokenValue<'a>> for TokenValues<'a> {
    fn from(value: TokenValue<'a>) -> Self {
        match value {
            TokenValue::Nothing => TokenValues::Nothing,
            t => TokenValues::Unit(t),
        }
    }
}

pub(crate) trait TokenAccumulator {
    fn token<'a>(&mut self, token: &'a str) -> TokenValues<'a>;
    fn process<'a>(&mut self, key_string: String) -> TokenValue<'a>;
    fn process_noop<'a>(&mut self, key_string: &'a str) -> TokenValue<'a>;
    fn flush<'a>(&mut self) -> TokenValues<'a>;
}

impl IndexAccumulator {
    pub(crate) fn new(
        dictionary: Vec<i64>,
        modifier: usize,
        formatter: Box<dyn Fn(usize, usize) -> String>,
    ) -> Self {
        Self {
            dictionary,
            token_buffer: vec![],
            is_accumulating: false,
            non_digits_followed_by_digits: Regex::new(NON_DIGITS_FOLLOWED_BY_DIGITS).unwrap(),
            formatter,
            modifier,
        }
    }

    fn clear_buffer(&mut self) -> String {
        let string = self.token_buffer.join("");
        self.is_accumulating = false;
        self.token_buffer.clear();
        string
    }

    fn push_buffer<S: ToString>(&mut self, token: S) {
        self.token_buffer.push(token.to_string());
        self.is_accumulating = true;
    }
}

impl TokenAccumulator for IndexAccumulator {
    fn token<'a>(&mut self, token: &'a str) -> TokenValues<'a> {
        if token.is_empty() {
            TokenValues::Nothing
        } else if token.parse::<i64>().is_ok() {
            self.push_buffer(token);
            TokenValues::Nothing
        } else if token.trim_end().parse::<i64>().is_ok() {
            if self.is_accumulating {
                self.push_buffer(token);
                let key_string = self.clear_buffer();
                self.process(key_string).into()
            } else {
                let key_string = self.clear_buffer();
                self.process(key_string).into()
            }
        } else if token.trim_start().parse::<i64>().is_ok() {
            if self.is_accumulating {
                let key_string = self.clear_buffer();
                let result = self.process(key_string);
                self.push_buffer(token);
                result.into()
            } else {
                let _key_string = self.clear_buffer();
                assert!(_key_string.is_empty());
                self.push_buffer(token);
                TokenValues::Nothing
            }
        } else if token.trim().parse::<i64>().is_ok() {
            if self.is_accumulating {
                let key_string = self.clear_buffer();
                let previous_result = self.process(key_string);
                let current_result = self.process(token.to_string());
                TokenValues::Twofer(previous_result, current_result)
            } else {
                let _key_string = self.clear_buffer();
                assert!(_key_string.is_empty());
                let current_result = self.process(token.to_string());
                TokenValues::Unit(current_result)
            }
        } else if self.is_accumulating {
            let key_string = self.clear_buffer();
            let result = self.process(key_string);
            TokenValues::Twofer(result, TokenValue::NoOp(token))
        } else {
            let _key_string = self.clear_buffer();
            assert!(_key_string.is_empty());
            TokenValues::Unit(TokenValue::NoOp(token))
        }
    }

    fn process<'a>(&mut self, key_string: String) -> TokenValue<'a> {
        if let Ok(key) = key_string.trim().parse::<i64>() {
            if let Some(value) = self.dictionary.iter().position(|i| *i == key) {
                let value_string = (self.formatter)(value, self.modifier);
                let value_string = key_string.replace(&key.to_string(), &value_string);
                TokenValue::Transform(value_string, value)
            } else {
                TokenValue::NoTransform(key_string)
            }
        } else {
            TokenValue::Nothing
        }
    }
    fn process_noop<'a>(&mut self, key_string: &'a str) -> TokenValue<'a> {
        if let Ok(key) = key_string.trim().parse::<i64>() {
            if let Some(value) = self.dictionary.iter().position(|i| *i == key) {
                let value_string = (self.formatter)(value, self.modifier);
                let value_string = key_string.replace(&key.to_string(), &value_string);
                TokenValue::Transform(value_string, value)
            } else {
                TokenValue::NoOp(key_string)
            }
        } else {
            TokenValue::Nothing
        }
    }

    fn flush<'a>(&mut self) -> TokenValues<'a> {
        let key_string = self.clear_buffer();
        self.process(key_string).into()
    }
}

#[cfg(test)]
mod test {
    use crate::inference::index_accumulator::TokenValue as TV;
    use crate::inference::index_accumulator::TokenValues as TVS;

    use super::{IndexAccumulator, TokenAccumulator};

    fn formatter(index: usize, modifier: usize) -> String {
        format!(
            "[{}](http://localhost/#{})",
            index + modifier,
            index + modifier
        )
    }

    #[test]
    fn empty() {
        let mut a = IndexAccumulator::new(vec![1234, 4321], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token(""));
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn plain_text() {
        let mut a = IndexAccumulator::new(vec![1234, 4321], 0, Box::new(formatter));

        assert_eq!(TVS::Unit(TV::NoOp("This")), a.token("This"));
        assert_eq!(TVS::Unit(TV::NoOp(" is")), a.token(" is"));
        assert_eq!(TVS::Unit(TV::NoOp(" a")), a.token(" a"));
        assert_eq!(TVS::Unit(TV::NoOp(" test")), a.token(" test"));
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched() {
        let mut a = IndexAccumulator::new(vec![1234, 4321], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(TVS::Nothing, a.token("4"));
        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Unit(TV::NoTransform("2341".to_string())), a.flush());
    }
    #[test]
    fn index_matched() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(TVS::Nothing, a.token("4"));
        assert_eq!(
            TVS::Unit(TV::Transform("[0](http://localhost/#0)".to_string(), 0)),
            a.flush()
        );
    }
    #[test]
    fn indices_unmatched() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(TVS::Unit(TV::NoTransform("123 ".to_string())), a.token(" "));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Unit(TV::NoTransform("321".to_string())), a.flush());
    }
    #[test]
    fn indices_matched_and_same() {
        let mut a = IndexAccumulator::new(vec![123], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(
            TVS::Twofer(
                TV::Transform("[0](http://localhost/#0)".to_string(), 0),
                TV::NoOp(" ")
            ),
            a.token(" ")
        );
        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(
            TVS::Unit(TV::Transform("[0](http://localhost/#0)".to_string(), 0)),
            a.flush()
        );
    }
    #[test]
    fn indices_matched_and_different() {
        let mut a = IndexAccumulator::new(vec![123, 321], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(
            TVS::Twofer(
                TV::Transform("[0](http://localhost/#0)".to_string(), 0),
                TV::NoOp(" ")
            ),
            a.token(" ")
        );
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(
            TVS::Unit(TV::Transform("[1](http://localhost/#1)".to_string(), 1),),
            a.flush()
        );
    }
    #[test]
    fn indices_single_matched() {
        let mut a = IndexAccumulator::new(vec![123], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(
            TVS::Twofer(
                TV::Transform("[0](http://localhost/#0)".to_string(), 0),
                TV::NoOp(" ")
            ),
            a.token(" ")
        );
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Unit(TV::NoTransform("321".to_string())), a.flush());
    }
    #[test]
    fn index_matched_leading() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token(" 1"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(TVS::Nothing, a.token("4"));
        assert_eq!(
            TVS::Unit(TV::Transform(" [0](http://localhost/#0)".to_string(), 0)),
            a.flush()
        );
    }
    #[test]
    fn index_matched_trailing() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(
            TVS::Unit(TV::Transform("[0](http://localhost/#0) ".to_string(), 0),),
            a.token("4 ")
        );
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn index_matched_leading_trailing() {
        let mut a = IndexAccumulator::new(vec![12], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token(" 1"));
        assert_eq!(
            TVS::Unit(TV::Transform(" [0](http://localhost/#0) ".to_string(), 0)),
            a.token("2 ")
        );
        assert_eq!(TVS::Nothing, a.flush());
    }

    #[test]
    fn index_matched_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token("1234"));
        assert_eq!(TVS::Nothing, a.token("56789"));
        assert_eq!(
            TVS::Unit(TV::Transform("[0](http://localhost/#0)".to_string(), 0)),
            a.flush()
        );
    }
    #[test]
    fn index_matched_leading_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token(" 1234"));
        assert_eq!(TVS::Nothing, a.token("56789"));
        assert_eq!(
            TVS::Unit(TV::Transform(" [0](http://localhost/#0)".to_string(), 0)),
            a.flush()
        );
    }
    #[test]
    fn index_matched_trailing_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token("1234"));
        assert_eq!(
            TVS::Unit(TV::Transform("123456789 ".to_string(), 0)),
            a.token("56789 ")
        );
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn index_matched_leading_trailing_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token(" 1234"));
        assert_eq!(
            TVS::Unit(TV::NoTransform(" 123456789 ".to_string())),
            a.token("56789 ")
        );
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched_leading() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(TVS::Nothing, a.token(" 1"));
        assert_eq!(TVS::Unit(TV::NoTransform(" 1".to_string())), a.token(" 2"));
        assert_eq!(TVS::Unit(TV::NoTransform(" 2".to_string())), a.token(" 3"));
        assert_eq!(TVS::Unit(TV::NoTransform(" 3".to_string())), a.token(" 4"));
        assert_eq!(TVS::Unit(TV::NoTransform(" 4".to_string())), a.flush());
    }
    #[test]
    fn index_unmatched_trailing() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(TVS::Unit(TV::NoTransform("1 ".to_string())), a.token("1 "));
        assert_eq!(TVS::Unit(TV::NoTransform("2 ".to_string())), a.token("2 "));
        assert_eq!(TVS::Unit(TV::NoTransform("3 ".to_string())), a.token("3 "));
        assert_eq!(TVS::Unit(TV::NoTransform("4 ".to_string())), a.token("4 "));
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched_leading_trailing() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(
            TVS::Unit(TV::NoTransform(" 1 ".to_string())),
            a.token(" 1 ")
        );
        assert_eq!(
            TVS::Unit(TV::NoTransform(" 2 ".to_string())),
            a.token(" 2 ")
        );
        assert_eq!(
            TVS::Unit(TV::NoTransform(" 3 ".to_string())),
            a.token(" 3 ")
        );
        assert_eq!(
            TVS::Unit(TV::NoTransform(" 4 ".to_string())),
            a.token(" 4 ")
        );
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched_leading_trailing_large_fragments() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(
            TVS::Unit(TV::NoTransform(" 1234 ".to_string())),
            a.token(" 1234 ")
        );
        assert_eq!(
            TVS::Unit(TV::NoTransform(" 123 ".to_string())),
            a.token(" 123 ")
        );
        assert_eq!(
            TVS::Unit(TV::NoTransform(" 12 ".to_string())),
            a.token(" 12 ")
        );
        assert_eq!(
            TVS::Unit(TV::NoTransform(" 34 ".to_string())),
            a.token(" 34 ")
        );
        assert_eq!(TVS::Nothing, a.flush());
    }
}
