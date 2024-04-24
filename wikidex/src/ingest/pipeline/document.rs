use chrono::NaiveDateTime;

pub(crate) struct Document {
    pub(crate) document: String,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}

pub(crate) struct DocumentWithHeading {
    pub(crate) document: String,
    pub(crate) heading: String,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}

#[derive(Default)]
pub(crate) struct CompressedDocument {
    pub(crate) document: Vec<u8>,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}
