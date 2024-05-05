use super::LlmMessage;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct LanguageServiceDocument {
    pub(crate) index: i64,
    pub(crate) text: String,
}
// pub(crate) struct LanguageServiceArguments<'arg> {
//     pub(crate) prompt: &'arg str,
// }
pub(crate) struct LanguageServiceArguments {
    pub(crate) messages: Vec<LlmMessage>,
    pub(crate) documents: Vec<LanguageServiceDocument>,
    pub(crate) user_query: String,
    pub(crate) max_tokens: u16,
    pub(crate) stop_phrases: Vec<String>,
}
