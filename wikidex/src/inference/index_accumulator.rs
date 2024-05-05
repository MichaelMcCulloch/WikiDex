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
    fn flush(&mut self) -> IndexAccumulatorReturn;
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
        if !self.is_accumulating {
            if token.trim().is_empty() {
                return IndexAccumulatorReturn::NoOp(token);
            }

            let token_as_number: Option<i64> = token.trim().parse().ok();

            if let Some(num) = token_as_number {
                if self.dictionary.contains(&num) {
                    self.is_accumulating = true;
                    self.token_buffer.push(token.to_string());
                    return IndexAccumulatorReturn::Nothing;
                }
            }
        } else {
            let token_as_number: Option<i64> = token.trim().parse().ok();

            if let Some(num) = token_as_number {
                if self.dictionary.contains(&num) {
                    self.token_buffer.push(token.to_string());
                    return IndexAccumulatorReturn::Nothing;
                }
            }

            // Flush the buffer if the current token does not continue the sequence
            let result = self.flush();
            if let IndexAccumulatorReturn::NoTransform(d) = result {
                // If the buffer was not transformed, return the result of the flush
                return IndexAccumulatorReturn::NoTransform(d);
            }
        }

        // If the token is not a number or not in the dictionary, return NoOp
        IndexAccumulatorReturn::NoOp(token)
    }

    fn flush(&mut self) -> IndexAccumulatorReturn {
        if self.token_buffer.is_empty() {
            return IndexAccumulatorReturn::Nothing;
        }

        let accumulated_string = self.token_buffer.join("");
        self.token_buffer.clear();
        self.is_accumulating = false;

        let accumulated_number: i64 = accumulated_string.parse().unwrap_or(0);

        if self.dictionary.contains(&accumulated_number) {
            let index = self
                .dictionary
                .iter()
                .position(|&x| x == accumulated_number)
                .unwrap();
            IndexAccumulatorReturn::Transform(index.to_string(), index)
        } else {
            IndexAccumulatorReturn::NoTransform(accumulated_string)
        }
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
    use crate::inference::index_accumulator::IndexAccumulatorReturn as I;

    use super::{IndexAccumulator, IndexAccumulatorTrait};

    #[test]
    fn plain_text() {
        let mut a = IndexAccumulator::new(vec![1234, 4321]);

        assert_eq!(I::NoOp("This"), a.token("This"));
        assert_eq!(I::NoOp(" is"), a.token(" is"));
        assert_eq!(I::NoOp(" a"), a.token(" a"));
        assert_eq!(I::NoOp(" test"), a.token(" test"));
        assert_eq!(I::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched() {
        let mut a = IndexAccumulator::new(vec![1234, 4321]);

        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("4"));
        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::NoTransform("2341".to_string()), a.flush());
    }
    #[test]
    fn index_matched() {
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("4"));
        assert_eq!(I::NoTransform("0".to_string()), a.flush());
    }
    #[test]
    fn indices_unmatched() {
        let mut a = IndexAccumulator::new(vec![1234]);

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
        let mut a = IndexAccumulator::new(vec![123]);

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
        let mut a = IndexAccumulator::new(vec![123, 321]);

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
        let mut a = IndexAccumulator::new(vec![123]);

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
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(I::Nothing, a.token(" 1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("4"));
        assert_eq!(I::NoTransform("0".to_string()), a.flush());
    }
    #[test]
    fn index_matched_trailing() {
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(I::Nothing, a.token("1"));
        assert_eq!(I::Nothing, a.token("2"));
        assert_eq!(I::Nothing, a.token("3"));
        assert_eq!(I::Nothing, a.token("4 "));
        assert_eq!(I::NoTransform("0".to_string()), a.flush());
    }
    #[test]
    fn index_matched_leading_trailing() {
        let mut a = IndexAccumulator::new(vec![12]);

        assert_eq!(I::Nothing, a.token(" 1"));
        assert_eq!(I::Transform(" 12 ".to_string(), 0), a.token("2 "));
        assert_eq!(I::Nothing, a.flush());
    }

    #[test]
    fn index_matched_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789]);

        assert_eq!(I::Nothing, a.token("1234"));
        assert_eq!(I::Nothing, a.token("56789"));
        assert_eq!(I::Transform("123456789".to_string(), 0), a.flush());
    }
    #[test]
    fn index_matched_leading_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789]);

        assert_eq!(I::Nothing, a.token(" 1234"));
        assert_eq!(I::Nothing, a.token("56789"));
        assert_eq!(I::Transform("123456789".to_string(), 0), a.flush());
    }
    #[test]
    fn index_matched_trailing_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789]);

        assert_eq!(I::Nothing, a.token("1234"));
        assert_eq!(I::Transform("123456789 ".to_string(), 0), a.token("56789 "));
        assert_eq!(I::Nothing, a.flush());
    }
    #[test]
    fn index_matched_leading_trailing_large_fragments() {
        let mut a = IndexAccumulator::new(vec![123456789]);

        assert_eq!(I::Nothing, a.token(" 1234"));
        assert_eq!(I::NoTransform(" 123456789 ".to_string()), a.token("56789 "));
        assert_eq!(I::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched_leading() {
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(I::Nothing, a.token(" 1"));
        assert_eq!(I::NoTransform(" 1".to_string()), a.token(" 2"));
        assert_eq!(I::NoTransform(" 2".to_string()), a.token(" 3"));
        assert_eq!(I::NoTransform(" 3".to_string()), a.token(" 4"));
        assert_eq!(I::NoTransform(" 4".to_string()), a.flush());
    }
    #[test]
    fn index_unmatched_trailing() {
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(I::NoTransform("1 ".to_string()), a.token("1 "));
        assert_eq!(I::NoTransform("2 ".to_string()), a.token("2 "));
        assert_eq!(I::NoTransform("3 ".to_string()), a.token("3 "));
        assert_eq!(I::NoTransform("4 ".to_string()), a.token("4 "));
        assert_eq!(I::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched_leading_trailing() {
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(I::NoTransform(" 1 ".to_string()), a.token(" 1 "));
        assert_eq!(I::NoTransform(" 2 ".to_string()), a.token(" 2 "));
        assert_eq!(I::NoTransform(" 3 ".to_string()), a.token(" 3 "));
        assert_eq!(I::NoTransform(" 4 ".to_string()), a.token(" 4 "));
        assert_eq!(I::Nothing, a.flush());
    }
    #[test]
    fn index_unmatched_leading_trailing_large_fragments() {
        let mut a = IndexAccumulator::new(vec![1234]);

        assert_eq!(I::NoTransform(" 1234 ".to_string()), a.token(" 1234 "));
        assert_eq!(I::NoTransform(" 123 ".to_string()), a.token(" 123 "));
        assert_eq!(I::NoTransform(" 12 ".to_string()), a.token(" 12 "));
        assert_eq!(I::NoTransform(" 34 ".to_string()), a.token(" 34 "));
        assert_eq!(I::Nothing, a.flush());
    }
}
