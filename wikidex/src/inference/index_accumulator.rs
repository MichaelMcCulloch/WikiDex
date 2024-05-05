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

pub(crate) trait TokenAccumulator {
    fn token<'a>(&mut self, token: &'a str) -> TokenValue<'a>;
    fn flush(&mut self) -> TokenValue;
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

    fn process_token<'a>(&mut self, token: &'a str) -> TokenValue<'a> {
        if let Ok(key) = token.parse::<i64>() {
            if let Some(value) = self.dictionary.iter().position(|i| *i == key) {
                TokenValue::Transform(token.replace(&key.to_string(), &value.to_string()), value)
            } else {
                self.is_accumulating = true;
                self.token_buffer.push(token.to_string());
                TokenValue::Nothing
            }
        } else if let Ok(key) = token.trim_start().parse::<i64>() {
            if let Some(value) = self.dictionary.iter().position(|i| *i == key) {
                TokenValue::Transform(token.replace(&key.to_string(), &value.to_string()), value)
            } else {
                self.is_accumulating = true;
                self.token_buffer.push(token.to_string());
                TokenValue::Nothing
            }
        } else if let Ok(key) = token.trim_end().parse::<i64>() {
            if let Some(value) = self.dictionary.iter().position(|i| *i == key) {
                TokenValue::Transform(token.replace(&key.to_string(), &value.to_string()), value)
            } else {
                TokenValue::NoOp(token)
            }
        } else if let Ok(key) = token.trim().parse::<i64>() {
            if let Some(value) = self.dictionary.iter().position(|i| *i == key) {
                TokenValue::Transform(token.replace(&key.to_string(), &value.to_string()), value)
            } else {
                TokenValue::NoOp(token)
            }
        } else {
            TokenValue::Nothing
        }
    }
}

impl TokenAccumulator for IndexAccumulator {
    fn token<'a>(&mut self, token: &'a str) -> TokenValue<'a> {
        if token.trim().parse::<i64>().is_ok() {
            self.token_buffer.push(token.to_string());
            self.is_accumulating = true;
            TokenValue::Nothing
        } else if self.is_accumulating {
            let key_string = self.token_buffer.join("");
            let key = key_string.trim().parse::<i64>().unwrap();

            let response = if let Some(value) = self.dictionary.iter().position(|s| *s == key) {
                let new_value = (self.formatter)(value, self.modifier);
                let key_string = key_string.replace(&key.to_string(), &new_value);
                let key_string = format!("{}{}", key_string, token);
                TokenValue::Transform(key_string, value)
            } else {
                TokenValue::NoTransform(key_string)
            };
            self.token_buffer.clear();
            self.is_accumulating = false;
            response
        } else {
            TokenValue::NoOp(token)
        }
    }

    fn flush(&mut self) -> TokenValue {
        TokenValue::Nothing
    }
}

// impl IndexAccumulatorTrait for IndexAccumulator {
//     fn token<'a>(&mut self, token: &'a str) -> IndexAccumulatorReturn<'a> {
//         if token.trim().parse::<i64>().is_ok() {
//             self.token_buffer.push(token.to_string());
//             self.is_accumulating = true;
//             IndexAccumulatorReturn::Nothing
//         } else if self.is_accumulating {
//             let index_string = self.token_buffer.join("");
//             let result = if let Ok(index) = index_string.trim().parse::<i64>() {
//                 if let Some(position) = self.dictionary.iter().position(|element| element == &index)
//                 {
//                     IndexAccumulatorReturn::Transform(
//                         index_string
//                             .replace(index.to_string().as_str(), position.to_string().as_str()),
//                         position,
//                     )
//                 } else {
//                     IndexAccumulatorReturn::NoTransform(index_string.to_string())
//                 }
//             } else {
//                 self.token_buffer.clear();
//                 IndexAccumulatorReturn::NoTransform(index_string)
//             };
//             self.token_buffer.clear();
//             self.is_accumulating = false;
//             result
//         } else {
//             IndexAccumulatorReturn::NoOp(token)
//         }
//     }
//     fn flush(&mut self) -> IndexAccumulatorReturn {
//         let string = self.token_buffer.join("");
//         self.token_buffer.clear();
//         if string.is_empty() {
//             IndexAccumulatorReturn::Nothing
//         } else {
//             IndexAccumulatorReturn::NoTransform(string)
//         }
//     }
// }

#[cfg(test)]
mod test {
    use crate::inference::index_accumulator::TokenValue as I;

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

        assert_eq!(I::Nothing, a.token(""));
        assert_eq!(I::Nothing, a.flush());
    }
    #[test]
    fn plain_text() {
        let mut a = IndexAccumulator::new(vec![1234, 4321], 0, Box::new(formatter));

        assert_eq!(I::NoOp("This"), a.token("This"));
        assert_eq!(I::NoOp(" is"), a.token(" is"));
        assert_eq!(I::NoOp(" a"), a.token(" a"));
        assert_eq!(I::NoOp(" test"), a.token(" test"));
        assert_eq!(I::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched() {
        let mut a = IndexAccumulator::new(vec![1234, 4321], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("4"));
        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::NoTransform("2341".to_string()), a.flush());
    }
    #[test]
    fn index_matched() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("4"));
        assert_eq!(I::NoTransform("0".to_string()), a.flush());
    }
    #[test]
    fn indices_unmatched() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::NoTransform("123 ".to_string()), a.token(" "));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::NoTransform("321".to_string()), a.flush());
    }
    #[test]
    fn indices_matched_and_same() {
        let mut a = IndexAccumulator::new(vec![123], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Transform("0 ".to_string(), 0), a.token(" "));
        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Transform("0".to_string(), 0), a.flush());
    }
    #[test]
    fn indices_matched_and_different() {
        let mut a = IndexAccumulator::new(vec![123, 321], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Transform("0 ".to_string(), 0), a.token(" "));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Transform("1".to_string(), 1), a.flush());
    }
    #[test]
    fn indices_single_matched() {
        let mut a = IndexAccumulator::new(vec![123], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Transform("0 ".to_string(), 0), a.token(" "));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::NoTransform("321".to_string()), a.flush());
    }
    #[test]
    fn index_matched_leading() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token(" 1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("4"));
        assert_eq!(I::NoTransform("0".to_string()), a.flush());
    }
    #[test]
    fn index_matched_trailing() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("4 "));
        assert_eq!(I::NoTransform("0".to_string()), a.flush());
    }
    #[test]
    fn index_matched_leading_trailing() {
        let mut a = IndexAccumulator::new(vec![12], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token(" 1"));
        assert_eq!(I::Transform(" 12 ".to_string(), 0), a.token("2 "));
        assert_eq!(I::Nothing, a.flush());
    }

    #[test]
    fn index_matched_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token("1234"));
        assert_eq!(I::Nothing, a.token("56789"));
        assert_eq!(I::Transform("123456789".to_string(), 0), a.flush());
    }
    #[test]
    fn index_matched_leading_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token(" 1234"));
        assert_eq!(I::Nothing, a.token("56789"));
        assert_eq!(I::Transform("123456789".to_string(), 0), a.flush());
    }
    #[test]
    fn index_matched_trailing_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token("1234"));
        assert_eq!(I::Transform("123456789 ".to_string(), 0), a.token("56789 "));
        assert_eq!(I::Nothing, a.flush());
    }
    #[test]
    fn index_matched_leading_trailing_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token(" 1234"));
        assert_eq!(I::NoTransform(" 123456789 ".to_string()), a.token("56789 "));
        assert_eq!(I::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched_leading() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(I::Nothing, a.token(" 1"));
        assert_eq!(I::NoTransform(" 1".to_string()), a.token(" 2"));
        assert_eq!(I::NoTransform(" 2".to_string()), a.token(" 3"));
        assert_eq!(I::NoTransform(" 3".to_string()), a.token(" 4"));
        assert_eq!(I::NoTransform(" 4".to_string()), a.flush());
    }
    #[test]
    fn index_unmatched_trailing() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(I::NoTransform("1 ".to_string()), a.token("1 "));
        assert_eq!(I::NoTransform("2 ".to_string()), a.token("2 "));
        assert_eq!(I::NoTransform("3 ".to_string()), a.token("3 "));
        assert_eq!(I::NoTransform("4 ".to_string()), a.token("4 "));
        assert_eq!(I::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched_leading_trailing() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(I::NoTransform(" 1 ".to_string()), a.token(" 1 "));
        assert_eq!(I::NoTransform(" 2 ".to_string()), a.token(" 2 "));
        assert_eq!(I::NoTransform(" 3 ".to_string()), a.token(" 3 "));
        assert_eq!(I::NoTransform(" 4 ".to_string()), a.token(" 4 "));
        assert_eq!(I::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched_leading_trailing_large_fragments() {
        let mut a = IndexAccumulator::new(vec![1234], 0, Box::new(formatter));

        assert_eq!(I::NoTransform(" 1234 ".to_string()), a.token(" 1234 "));
        assert_eq!(I::NoTransform(" 123 ".to_string()), a.token(" 123 "));
        assert_eq!(I::NoTransform(" 12 ".to_string()), a.token(" 12 "));
        assert_eq!(I::NoTransform(" 34 ".to_string()), a.token(" 34 "));
        assert_eq!(I::Nothing, a.flush());
    }
}
