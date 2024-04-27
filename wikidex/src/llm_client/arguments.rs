pub(crate) struct LanguageServiceArguments<'arg> {
    pub(crate) prompt: &'arg str,
    pub(crate) query: &'arg str,
    pub(crate) indices: &'arg Vec<i64>,
}
