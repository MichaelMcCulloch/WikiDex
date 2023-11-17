use regex::Regex;

const CITATION_REGEX: &str = "(C|c)ite|(C|c)itation";
const SFN_REGEX: &str = "(S|s)fn";
const SFNM_REGEX: &str = "(S|s)fnm";

const BOOK_REGEX: &str = "(B|b)ook";
const ENCYCLOPEDIA_REGEX: &str = "(E|e)ncyclopedia";
const JOURNAL_REGEX: &str = "(J|j)ournal";
const MAGAZINE_REGEX: &str = "(M|m)agazine";
const NEWS_REGEX: &str = "(N|n)ews";
const WEB_REGEX: &str = "(W|w)eb";
const REFN_REGEX: &str = "(R|r)efn";
const LANGUAGE_REGEX: &str = "(L|l)ang";
const LINKTEXT_REGEX: &str = "(L|l)inktext";
const TWO_MORE_NEWLINES: &str = "(\n{2,})";

#[derive(Clone)]
pub(crate) struct Regexes {
    pub(crate) citation: Regex,
    pub(crate) sfn: Regex,
    pub(crate) sfnm: Regex,
    pub(crate) book: Regex,
    pub(crate) encyclopedia: Regex,
    pub(crate) journal: Regex,
    pub(crate) magazine: Regex,
    pub(crate) news: Regex,
    pub(crate) web: Regex,
    pub(crate) refn: Regex,
    pub(crate) language: Regex,
    pub(crate) linktext: Regex,
    pub(crate) newlines: Regex,
}

impl Regexes {
    pub(crate) fn new() -> Regexes {
        Regexes {
            citation: Regex::new(CITATION_REGEX).unwrap(),
            sfn: Regex::new(SFN_REGEX).unwrap(),
            sfnm: Regex::new(SFNM_REGEX).unwrap(),
            book: Regex::new(BOOK_REGEX).unwrap(),
            encyclopedia: Regex::new(ENCYCLOPEDIA_REGEX).unwrap(),
            journal: Regex::new(JOURNAL_REGEX).unwrap(),
            magazine: Regex::new(MAGAZINE_REGEX).unwrap(),
            news: Regex::new(NEWS_REGEX).unwrap(),
            web: Regex::new(WEB_REGEX).unwrap(),
            refn: Regex::new(REFN_REGEX).unwrap(),
            language: Regex::new(LANGUAGE_REGEX).unwrap(),
            linktext: Regex::new(LINKTEXT_REGEX).unwrap(),
            newlines: Regex::new(TWO_MORE_NEWLINES).unwrap(),
        }
    }
}
