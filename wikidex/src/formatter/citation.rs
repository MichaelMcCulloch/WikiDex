use super::style::CitationStyle;

pub(crate) trait Cite {
    fn format(&self, style: &CitationStyle) -> String;
    fn url(&self) -> String;
    fn title(&self) -> String;
}
