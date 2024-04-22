use chrono::NaiveDateTime;

pub(crate) struct Document {
    pub(crate) document: String,
    pub(crate) article_title: String,
    pub(crate) access_date: NaiveDateTime,
    pub(crate) modification_date: NaiveDateTime,
}
