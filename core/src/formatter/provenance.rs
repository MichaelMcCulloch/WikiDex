use super::{CitationStyle, Cite};

use chrono::NaiveDate;
use redis::{FromRedisValue, RedisError, RedisResult, ToRedisArgs, Value};
use rkyv::{archived_root, Archive, Deserialize, Infallible, Serialize};

type WikipediaArticleTitle = String;
type AccessDate = NaiveDate;
type LastModificationDate = NaiveDate;

#[derive(Clone, Serialize, Deserialize, Archive)]
pub(crate) enum Provenance {
    Wikipedia(WikipediaArticleTitle, AccessDate, LastModificationDate),
}

impl FromRedisValue for Provenance {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        if let Value::Data(bytes) = v {
            let archived = unsafe { archived_root::<Provenance>(bytes.as_slice()) };
            archived.deserialize(&mut Infallible).map_err(|_| {
                RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed"))
            })
        } else {
            Err(RedisError::from((
                redis::ErrorKind::TypeError,
                "Expected bytes, got something else",
            )))
        }
    }
}

impl ToRedisArgs for Provenance {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let bytes = rkyv::to_bytes::<_, 1024>(self).unwrap();
        out.write_arg(&bytes);
    }
}

impl Cite for Provenance {
    fn format(&self, style: &CitationStyle) -> String {
        match self {
            Provenance::Wikipedia(title, access_date, edit_date) => match style {
                CitationStyle::Chigago => {
                    let article_url = self.url();
                    let access_date = access_date.format("%-d %B %Y");
                    let edit_date = edit_date.format("%-d %B %Y");
                    format!("\"{title}\" Wikipedia. Last modified {edit_date}, Accessed {access_date}, {article_url}.")
                }
                CitationStyle::MLA => {
                    let article_url = self.url();
                    let access_date = access_date.format("%-d %B %Y");
                    let edit_date = edit_date.format("%-d %B %Y");
                    format!("\"{title}\" Wikipedia, Wikimedia Foundation, {edit_date}, {article_url}. Accessed {access_date}.")
                }
                CitationStyle::APA => {
                    let article_url = self.url();
                    let access_date = access_date.format("%B %-d, %Y");
                    let edit_date = edit_date.format("%Y, %B %-d");
                    format!("{title}. {edit_date}. In Wikipedia. Retrieved {access_date}, from {article_url}")
                }
            },
        }
    }

    fn url(&self) -> String {
        match self {
            Provenance::Wikipedia(title, _, _) => {
                format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"))
            }
        }
    }

    fn title(&self) -> String {
        match self {
            Provenance::Wikipedia(title, _, _) => title.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn wiki_mla() {
        let expected = r#""Austrian German" Wikipedia, Wikimedia Foundation, 1 October 2023, https://en.wikipedia.org/wiki/Austrian_German. Accessed 1 October 2023."#;

        let provenance = Provenance::Wikipedia(
            "Austrian German".to_string(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
        );

        assert_eq!(expected, provenance.format(&CitationStyle::MLA))
    }
    #[test]
    fn wiki_apa() {
        let expected = r#"Austrian German. 2023, October 1. In Wikipedia. Retrieved October 1, 2023, from https://en.wikipedia.org/wiki/Austrian_German"#;

        let provenance = Provenance::Wikipedia(
            "Austrian German".to_string(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
        );

        assert_eq!(expected, provenance.format(&CitationStyle::APA))
    }
    #[test]
    fn wiki_chicago() {
        let expected = r#""Austrian German" Wikipedia. Last modified 1 October 2023, Accessed 1 October 2023, https://en.wikipedia.org/wiki/Austrian_German."#;

        let provenance = Provenance::Wikipedia(
            "Austrian German".to_string(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
        );

        assert_eq!(expected, provenance.format(&CitationStyle::Chigago))
    }
}
