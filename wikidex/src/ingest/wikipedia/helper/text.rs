use std::collections::VecDeque;

pub(crate) struct RecursiveCharacterTextSplitter<'a> {
    chunk_size: usize,
    chunk_overlap: usize,
    separators: Vec<&'a str>,
    keep_separator: bool,
}

impl<'a> RecursiveCharacterTextSplitter<'a> {
    pub fn new(
        chunk_size: usize,
        chunk_overlap: usize,
        separators: Option<Vec<&'a str>>,
        keep_separator: bool,
    ) -> Self {
        RecursiveCharacterTextSplitter {
            chunk_size,
            chunk_overlap,
            separators: separators.unwrap_or_else(|| vec![&"\n\n", &"\n", &" ", &""]),
            keep_separator,
        }
    }

    fn split_text_with_separator(&self, text: &str, separator: &str) -> Vec<String> {
        let mut results = Vec::new();
        let mut last = 0;

        for (start, part) in text.match_indices(separator) {
            if self.keep_separator {
                results.push(text[last..start + part.len()].to_string());
            } else {
                results.push(text[last..start].to_string());
            }
            last = start + part.len();
        }

        if last < text.len() {
            results.push(text[last..].to_string());
        }

        results
    }

    fn recursive_split(&self, text: &str, separator_index: usize) -> Vec<String> {
        if separator_index >= self.separators.len() {
            return vec![text.to_string()];
        }

        let separator = &self.separators[separator_index];
        let parts = if separator.is_empty() {
            text.chars().map(|c| c.to_string()).collect()
        } else {
            self.split_text_with_separator(text, separator)
        };

        let mut chunks = Vec::new();
        let mut buffer = VecDeque::new();

        for part in parts {
            if part.len() >= self.chunk_size {
                if !buffer.is_empty() {
                    chunks.push(self.merge_buffer(&mut buffer));
                }
                chunks.extend(self.recursive_split(&part, separator_index + 1));
                continue;
            }

            buffer.push_back(part);
            if buffer.iter().map(String::len).sum::<usize>() >= self.chunk_size {
                chunks.push(self.merge_buffer(&mut buffer));
            }
        }

        if !buffer.is_empty() {
            chunks.push(self.merge_buffer(&mut buffer));
        }

        chunks
    }

    fn merge_buffer(&self, buffer: &mut VecDeque<String>) -> String {
        let mut merged = String::new();
        while let Some(chunk) = buffer.pop_front() {
            merged.push_str(&chunk);
            if merged.len() >= self.chunk_size - self.chunk_overlap {
                break;
            }
        }
        merged
    }

    pub fn split_text(&self, text: &str) -> Vec<String> {
        self.recursive_split(text, 0)
    }
}

#[cfg(test)]
mod tests_text_splitter {

    use crate::{
        ingest::wikipedia::{
            helper::text::RecursiveCharacterTextSplitter, markup_processor::Process,
            WikiMarkupProcessor,
        },
        test_data::SUPREME_COURT_VOL_129,
    };

    #[test]
    fn read_document_file_to_string() {
        std::env::set_var("RUST_LOG", "info");
        env_logger::init();

        let document_text = SUPREME_COURT_VOL_129;

        let processor = WikiMarkupProcessor::new();
        let process = processor.process(document_text).unwrap();
        let split = RecursiveCharacterTextSplitter::new(1024, 128, None, true);
        let splits = split.split_text(&process);
        println!("{splits:#?}")
    }
}
