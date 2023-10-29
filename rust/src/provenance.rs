use crate::formatter::{citation::Cite, style::CitationStyle};
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
        let provenance = Provenance::Wikipedia(
            "Austrian German".to_string(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
            NaiveDate::from_ymd_opt(2023, 10, 01).unwrap(),
        );

        let f = provenance.format(CitationStyle::Chigago);
        println!("{f}");
        let f = provenance.format(CitationStyle::MLA);
        println!("{f}");
        let f = provenance.format(CitationStyle::APA);
        println!("{f}")
    }
}
