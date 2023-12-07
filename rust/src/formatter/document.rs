pub(crate) trait TextFormatter {
    fn format_document(document_ordinal: usize, document_text: &str) -> String {
        format!("[{document_ordinal}]:\n{document_text}")
    }
}

pub(crate) struct DocumentFormatter;

impl TextFormatter for DocumentFormatter {}
