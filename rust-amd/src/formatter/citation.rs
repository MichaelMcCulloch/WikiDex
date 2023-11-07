use super::style::CitationStyle;

pub(crate) trait Cite {
    fn format(&self, style: CitationStyle) -> String;
}
