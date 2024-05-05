use crate::docstore::Document;

use super::LlmMessage;

// pub(crate) struct LanguageServiceArguments<'arg> {
//     pub(crate) prompt: &'arg str,
// }
pub(crate) struct LanguageServiceArguments {
    pub(crate) messages: Vec<LlmMessage>,
    pub(crate) documents: Vec<Document>,
    pub(crate) user_query: String,
}
