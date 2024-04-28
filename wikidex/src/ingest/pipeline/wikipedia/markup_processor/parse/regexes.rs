use regex::Regex;

const CITATION_REGEX: &str = "(C|c)ite|(C|c)itation";
const SFNM_REGEX: &str = "(S|s)fnm";
const SFN_REGEX: &str = "(S|s)fn";

const BOOK_REGEX: &str = "(B|b)ook";
const ENCYCLOPEDIA_REGEX: &str = "(E|e)ncyclopedia";
const MAGAZINE_REGEX: &str = "(M|m)agazine";
const JOURNAL_REGEX: &str = "(J|j)ournal";
const NEWS_REGEX: &str = "(N|n)ews";
const WEB_REGEX: &str = "(W|w)eb";
const REFN_REGEX: &str = "(R|r)efn";
const LANGUAGE_REGEX: &str = "(L|l)ang";
const LINKTEXT_REGEX: &str = "(L|l)inktext";
const THREE_OR_MORE_NEWLINES: &str = "(\\s{3,})";
const TWO_OR_MORE_WHITESPACES: &str = "([ ]{2,})";
const SPACE_COMMA: &str = "([ ]*,)";
const SPACE_PERIOD: &str = "([ ]*\\.)";
const PILCROW: &str = "([ ]*¶[ ]*)";

#[derive(Clone)]
pub(crate) struct Regexes {
    pub(crate) _citation: Regex,
    pub(crate) _sfn: Regex,
    pub(crate) _sfnm: Regex,
    pub(crate) _book: Regex,
    pub(crate) _encyclopedia: Regex,
    pub(crate) _journal: Regex,
    pub(crate) _magazine: Regex,
    pub(crate) _news: Regex,
    pub(crate) _web: Regex,
    pub(crate) refn: Regex,
    pub(crate) language: Regex,
    pub(crate) linktext: Regex,
    pub(crate) threelines: Regex,
    pub(crate) twospace: Regex,
    pub(crate) space_coma: Regex,
    pub(crate) space_period: Regex,
    pub(crate) pilcrow: Regex,
}

impl Regexes {
    pub(crate) fn new() -> Regexes {
        Regexes {
            _citation: Regex::new(CITATION_REGEX).unwrap(),
            _sfn: Regex::new(SFN_REGEX).unwrap(),
            _sfnm: Regex::new(SFNM_REGEX).unwrap(),
            _book: Regex::new(BOOK_REGEX).unwrap(),
            _encyclopedia: Regex::new(ENCYCLOPEDIA_REGEX).unwrap(),
            _journal: Regex::new(JOURNAL_REGEX).unwrap(),
            _magazine: Regex::new(MAGAZINE_REGEX).unwrap(),
            _news: Regex::new(NEWS_REGEX).unwrap(),
            _web: Regex::new(WEB_REGEX).unwrap(),
            refn: Regex::new(REFN_REGEX).unwrap(),
            language: Regex::new(LANGUAGE_REGEX).unwrap(),
            linktext: Regex::new(LINKTEXT_REGEX).unwrap(),
            threelines: Regex::new(THREE_OR_MORE_NEWLINES).unwrap(),
            twospace: Regex::new(TWO_OR_MORE_WHITESPACES).unwrap(),
            space_coma: Regex::new(SPACE_COMMA).unwrap(),
            space_period: Regex::new(SPACE_PERIOD).unwrap(),
            pilcrow: Regex::new(PILCROW).unwrap(),
        }
    }
}
