use bytes::Bytes;
use std::error::Error;
use tokio::sync::mpsc::UnboundedSender;

use crate::server::{Conversation, Message};

#[async_trait::async_trait]
pub(crate) trait QueryEngine {
    type E: Error;
    async fn query(&self, question: &str) -> Result<String, Self::E>;
    async fn conversation(&self, conversation: &Conversation) -> Result<Message, Self::E>;
    async fn streaming_conversation(
        &self,
        conversation: &Conversation,
        tx: UnboundedSender<Bytes>,
    ) -> Result<(), Self::E>;
}
