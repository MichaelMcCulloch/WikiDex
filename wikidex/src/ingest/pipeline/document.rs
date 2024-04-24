use std::fmt::Display;

use chrono::NaiveDateTime;
#[derive(Clone)]
pub(crate) struct Document {
    pub(crate) document: String,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}
#[derive(Clone, Default)]
pub(crate) struct DocumentHeading {
    pub(crate) document: String,
    pub(crate) heading: String,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}

#[derive(Clone)]
pub(crate) struct DocumentHeadingEmbedding {
    pub(crate) document: String,
    pub(crate) heading: String,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}

#[derive(Clone, Default)]
pub(crate) struct DocumentCompressed {
    pub(crate) document: Vec<u8>,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}

impl Display for DocumentHeading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n\n{}", self.heading, self.document)
    }
}
