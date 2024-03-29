pub(crate) trait TextFormatter {
    fn format_document(
        document_ordinal: usize,
        _document_title: &str,
        document_text: &str,
    ) -> String {
        format!("1. ```{document_ordinal}\n{document_text}\n```")
    }
}

pub(crate) struct DocumentFormatter;

impl TextFormatter for DocumentFormatter {}
