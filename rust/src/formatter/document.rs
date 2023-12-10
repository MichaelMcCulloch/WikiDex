pub(crate) trait TextFormatter {
    fn format_document(document_title: &str, document_text: &str) -> String {
        format!("```{document_title}\n{document_text}\n```")
    }
}

pub(crate) struct DocumentFormatter;

impl TextFormatter for DocumentFormatter {}
