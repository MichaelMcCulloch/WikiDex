pub(crate) trait TextFormatter {
    fn format_document(document_ordinal: usize, document_text: &str) -> String {
        format!("BEGIN DOCUMENT {document_ordinal}\n§§§\n{document_text}\n§§§\nEND DOCUMENT {document_ordinal}")
    }
}

pub(crate) struct DocumentFormatter;

impl TextFormatter for DocumentFormatter {}
