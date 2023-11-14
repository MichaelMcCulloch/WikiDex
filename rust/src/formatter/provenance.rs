use super::{CitationStyle, Cite};

use chrono::NaiveDate;

type WikipediaArticleTitle = String;
type AccessDate = NaiveDate;
type LastModificationDate = NaiveDate;

#[derive(Clone)]
pub(crate) enum Provenance {
    Wikipedia(WikipediaArticleTitle, AccessDate, LastModificationDate),
}

impl Cite for Provenance {
    fn format(&self, style: CitationStyle) -> String {
        match self {
            Provenance::Wikipedia(title, access_date, edit_date) => match style {
                CitationStyle::Chigago => {
                    let url_safe_title = title.replace(" ", "_");
                    let access_date = access_date.format("%-d %B %Y");
                    let edit_date = edit_date.format("%-d %B %Y");
                    format!("\"{title}\" Wikipedia. Last modified {edit_date}, Accessed {access_date}, https://en.wikipedia.org/wiki/{url_safe_title}.")
                }
                CitationStyle::MLA => {
                    let url_safe_title = title.replace(" ", "_");
                    let access_date = access_date.format("%-d %B %Y");
                    let edit_date = edit_date.format("%-d %B %Y");
                    format!("\"{title}\" Wikipedia, Wikimedia Foundation, {edit_date}, https://en.wikipedia.org/wiki/{url_safe_title}. Accessed {access_date}.")
                }
                CitationStyle::APA => {
                    let url_safe_title = title.replace(" ", "_");
                    let access_date = access_date.format("%B %-d, %Y");
                    let edit_date = edit_date.format("%Y, %B %-d");
                    format!("{title}. {edit_date}. In Wikipedia. Retrieved {access_date}, from https://en.wikipedia.org/wiki/{url_safe_title}")
                }
            },
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

        assert_eq!(expected, provenance.format(CitationStyle::MLA))
    }
    #[test]
    fn wiki_apa() {
        let expected = r#"Austrian German. 2023, October 1. In Wikipedia. Retrieved October 1, 2023, from https://en.wikipedia.org/wiki/Austrian_German"#;

        let provenance = Provenance::Wikipedia(
            "Austrian German".to_string(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
        );

        assert_eq!(expected, provenance.format(CitationStyle::APA))
    }
    #[test]
    fn wiki_chicago() {
        let expected = r#""Austrian German" Wikipedia. Last modified 1 October 2023, Accessed 1 October 2023, https://en.wikipedia.org/wiki/Austrian_German."#;

        let provenance = Provenance::Wikipedia(
            "Austrian German".to_string(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
        );

        assert_eq!(expected, provenance.format(CitationStyle::Chigago))
    }
}
