use std::fmt::Display;

pub(crate) enum CitationStyle {
    Chigago,
    MLA,
    APA,
}

impl Display for CitationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CitationStyle::Chigago => write!(f, "Chigago"),
            CitationStyle::MLA => write!(f, "MLA"),
            CitationStyle::APA => write!(f, "APA"),
        }
    }
}
