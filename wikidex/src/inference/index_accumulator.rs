const NON_DIGITS_FOLLOWED_BY_DIGITS: &str = r#"^(\D*)(\d+)$"#;

pub(crate) struct IndexAccumulator {
    dictionary: Vec<i64>,
    token_buffer: Vec<String>,
    is_accumulating: bool,
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
    pub(crate) fn new(dictionary: Vec<i64>) -> Self {
        Self {
            dictionary,
            token_buffer: vec![],
            is_accumulating: false,
        }
    }
}

impl TokenAccumulator for IndexAccumulator {
    fn token<'a>(&mut self, token: &'a str) -> TokenValues<'a> {
        if token.is_empty() {
            TokenValues::Nothing
        } else if token.parse::<i64>().is_ok() {
            self.token_buffer.push(token.to_string());
            self.is_accumulating = true;
            TokenValues::Nothing
        } else if token.trim_end().parse::<i64>().is_ok() {
            if self.is_accumulating {
                self.token_buffer.push(token.to_string());
                let key_string = self.token_buffer.join("");
                self.is_accumulating = false;
                self.token_buffer.clear();
                self.process(key_string).into()
            } else {
                self.process_noop(token).into()
            }
        } else if token.trim_start().parse::<i64>().is_ok() {
            if self.is_accumulating {
                let key_string = self.token_buffer.join("");
                self.token_buffer.clear();
                let result = self.process(key_string);
                self.token_buffer.push(token.to_string());
                self.is_accumulating = true;
                result.into()
            } else {
                self.token_buffer.clear();
                self.token_buffer.push(token.to_string());
                self.is_accumulating = true;
                TokenValues::Nothing
            }
        } else if token.trim().parse::<i64>().is_ok() {
            if self.is_accumulating {
                let key_string = self.token_buffer.join("");
                self.is_accumulating = false;
                self.token_buffer.clear();
                let previous_result = self.process(key_string);
                let current_result = self.process(token.to_string());
                TokenValues::Twofer(previous_result, current_result)
            } else {
                self.is_accumulating = false;
                self.token_buffer.clear();
                let current_result = self.process(token.to_string());
                TokenValues::Unit(current_result)
            }
        } else if self.is_accumulating {
            let key_string = self.token_buffer.join("");
            self.is_accumulating = false;
            self.token_buffer.clear();
            let result = self.process(key_string);
            TokenValues::Twofer(result, TokenValue::NoOp(token))
        } else {
            let key_string = self.token_buffer.join("");
            self.is_accumulating = false;
            self.token_buffer.clear();
            assert!(key_string.is_empty());
            TokenValues::Unit(TokenValue::NoOp(token))
        }
    }

    fn process<'a>(&mut self, key_string: String) -> TokenValue<'a> {
        if let Ok(key) = key_string.trim().parse::<i64>() {
            if let Some(value) = self.dictionary.iter().position(|i| *i == key) {
                TokenValue::Transform(key_string, value)
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
                TokenValue::Transform(key_string.to_string(), value)
            } else {
                TokenValue::NoOp(key_string)
            }
        } else {
            TokenValue::Nothing
        }
    }

    fn flush<'a>(&mut self) -> TokenValues<'a> {
        let key_string = {
            let string = self.token_buffer.join("");
            self.is_accumulating = false;
            self.token_buffer.clear();
            string
        };
        self.process(key_string).into()
    }
}

#[cfg(test)]
mod test {
    use crate::inference::index_accumulator::TokenValue as TV;
    use crate::inference::index_accumulator::TokenValues as TVS;

    use super::{IndexAccumulator, TokenAccumulator};

    #[test]
    fn empty() {
        let mut a = IndexAccumulator::new(vec![1234, 4321]);

        assert_eq!(TVS::Nothing, a.token(""));
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn plain_text() {
        let mut a = IndexAccumulator::new(vec![1234, 4321]);

        assert_eq!(TVS::Unit(TV::NoOp("This")), a.token("This"));
        assert_eq!(TVS::Unit(TV::NoOp(" is")), a.token(" is"));
        assert_eq!(TVS::Unit(TV::NoOp(" a")), a.token(" a"));
        assert_eq!(TVS::Unit(TV::NoOp(" test")), a.token(" test"));
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched() {
        let mut a = IndexAccumulator::new(vec![1234, 4321]);

        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(TVS::Nothing, a.token("4"));
        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Unit(TV::NoTransform("2341".to_string())), a.flush());
    }
    #[test]
    fn index_matched() {
        let mut a = IndexAccumulator::new(vec![1234]);

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
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(
            TVS::Twofer(TV::NoTransform("123".to_string()), TV::NoOp(" ")),
            a.token(" ")
        );
        assert_eq!(TVS::Nothing, a.token("3"));
        assert_eq!(TVS::Nothing, a.token("2"));
        assert_eq!(TVS::Nothing, a.token("1"));
        assert_eq!(TVS::Unit(TV::NoTransform("321".to_string())), a.flush());
    }
    #[test]
    fn indices_matched_and_same() {
        let mut a = IndexAccumulator::new(vec![123]);

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
        let mut a = IndexAccumulator::new(vec![123, 321]);

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
        let mut a = IndexAccumulator::new(vec![123]);

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
        let mut a = IndexAccumulator::new(vec![1234]);

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
        let mut a = IndexAccumulator::new(vec![1234]);

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
        let mut a = IndexAccumulator::new(vec![12]);

        assert_eq!(TVS::Nothing, a.token(" 1"));
        assert_eq!(
            TVS::Unit(TV::Transform(" [0](http://localhost/#0) ".to_string(), 0)),
            a.token("2 ")
        );
        assert_eq!(TVS::Nothing, a.flush());
    }

    #[test]
    fn index_matched_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789]);

        assert_eq!(TVS::Nothing, a.token("1234"));
        assert_eq!(TVS::Nothing, a.token("56789"));
        assert_eq!(
            TVS::Unit(TV::Transform("[0](http://localhost/#0)".to_string(), 0)),
            a.flush()
        );
    }
    #[test]
    fn index_matched_leading_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789]);

        assert_eq!(TVS::Nothing, a.token(" 1234"));
        assert_eq!(TVS::Nothing, a.token("56789"));
        assert_eq!(
            TVS::Unit(TV::Transform(" [0](http://localhost/#0)".to_string(), 0)),
            a.flush()
        );
    }
    #[test]
    fn index_matched_trailing_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789]);

        assert_eq!(TVS::Nothing, a.token("1234"));
        assert_eq!(
            TVS::Unit(TV::Transform("[0](http://localhost/#0) ".to_string(), 0)),
            a.token("56789 ")
        );
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn index_matched_leading_trailing_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789]);

        assert_eq!(TVS::Nothing, a.token(" 1234"));
        assert_eq!(
            TVS::Unit(TV::Transform(" [0](http://localhost/#0) ".to_string(), 0)),
            a.token("56789 ")
        );
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched_leading() {
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(TVS::Nothing, a.token(" 1"));
        assert_eq!(TVS::Unit(TV::NoTransform(" 1".to_string())), a.token(" 2"));
        assert_eq!(TVS::Unit(TV::NoTransform(" 2".to_string())), a.token(" 3"));
        assert_eq!(TVS::Unit(TV::NoTransform(" 3".to_string())), a.token(" 4"));
        assert_eq!(TVS::Unit(TV::NoTransform(" 4".to_string())), a.flush());
    }
    #[test]
    fn index_unmatched_trailing() {
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(TVS::Unit(TV::NoOp("1 ")), a.token("1 "));
        assert_eq!(TVS::Unit(TV::NoOp("2 ")), a.token("2 "));
        assert_eq!(TVS::Unit(TV::NoOp("3 ")), a.token("3 "));
        assert_eq!(TVS::Unit(TV::NoOp("4 ")), a.token("4 "));
        assert_eq!(TVS::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched_leading_trailing() {
        let mut a: IndexAccumulator = IndexAccumulator::new(vec![1234]);

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
        let mut a = IndexAccumulator::new(vec![1234]);

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

    #[test]
    fn index_unmatched_letters_large_fragments() {
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(TVS::Unit(TV::NoOp("i123i")), a.token("i123i"));
        assert_eq!(TVS::Unit(TV::NoOp("i12i")), a.token("i12i"));
        assert_eq!(TVS::Unit(TV::NoOp("i34i")), a.token("i34i"));
        assert_eq!(TVS::Nothing, a.flush());
    }
}
