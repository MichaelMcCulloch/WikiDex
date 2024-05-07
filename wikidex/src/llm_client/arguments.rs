use super::LlmMessage;
use serde::Serialize;

#[derive(Serialize)]
pub struct LanguageServiceDocument {
    pub index: i64,
    pub text: String,
}
// pub(crate) struct LanguageServiceArguments<'arg> {
//     pub(crate) prompt: &'arg str,
// }
pub struct LanguageServiceArguments {
    pub messages: Vec<LlmMessage>,
    pub documents: Vec<LanguageServiceDocument>,
    pub user_query: String,
    pub max_tokens: u16,
    pub stop_phrases: Vec<String>,
}
