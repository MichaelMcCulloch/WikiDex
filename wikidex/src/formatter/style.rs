use std::fmt::Display;

#[allow(dead_code)]
pub(crate) enum CitationStyle {
    Chigago,
    Mla,
    Apa,
}

impl Display for CitationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CitationStyle::Chigago => write!(f, "Chigago"),
            CitationStyle::Mla => write!(f, "MLA"),
            CitationStyle::Apa => write!(f, "APA"),
        }
    }
}
