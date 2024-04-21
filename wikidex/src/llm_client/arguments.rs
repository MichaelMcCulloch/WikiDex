pub(crate) struct LlmArgSource {
    pub(crate) index: i64,
}

pub(crate) struct LanguageServiceArguments<'arg> {
    pub(crate) system: &'arg str,
    pub(crate) documents: &'arg str,
    pub(crate) query: &'arg str,
    pub(crate) indices: &'arg Vec<i64>,
}
