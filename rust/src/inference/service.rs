use std::error::Error;

use crate::server::{Conversation, Message};

#[async_trait::async_trait]
pub(crate) trait QueryEngine {
    type E: Error;
    async fn query(&self, question: &str) -> Result<String, Self::E>;
    async fn conversation(&self, conversation: &Conversation) -> Result<Message, Self::E>;
}
