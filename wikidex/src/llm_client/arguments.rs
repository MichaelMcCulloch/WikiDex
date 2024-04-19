use crate::server::Source;

pub(crate) struct LanguageServiceArguments<'arg> {
    pub(crate) system: &'arg str,
    pub(crate) documents: &'arg str,
    pub(crate) query: &'arg str,
    pub(crate) sources: &'arg Vec<Source>,
}
