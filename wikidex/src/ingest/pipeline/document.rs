use std::fmt::Display;

use chrono::NaiveDateTime;
#[derive(Default, Clone)]
pub(crate) struct Document {
    pub(crate) document: String,
    pub(crate) article_title: String,
    pub(crate) article_id: i64,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}

#[derive(Default, Clone)]
pub(crate) struct DocumentHeading {
    pub(crate) document: String,
    pub(crate) heading: String,
    pub(crate) document_id: i64,
    pub(crate) article_id: i64,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}

#[derive(Default, Clone)]
pub(crate) struct DocumentTextHeadingEmbedding {
    pub(crate) text: String,
    pub(crate) heading: String,
    pub(crate) document_id: i64,
    pub(crate) article_id: i64,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
    pub(crate) embedding: Vec<f32>,
}

#[derive(Default, Clone)]
pub(crate) struct DocumentCompressed {
    pub(crate) document: Vec<u8>,
    pub(crate) heading: String,
    pub(crate) document_id: i64,
    pub(crate) article_id: i64,
    pub(crate) embedding: Vec<f32>,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}

impl Display for DocumentHeading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n\n{}", self.heading, self.document)
    }
}
