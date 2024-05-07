use std::collections::HashMap;

use bytes::Bytes;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::formatter::{Cite};

// type Source = (String, String, String, String);
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub(crate) struct Source {
    pub(crate) index: i64,
    pub(crate) citation: String,
    pub(crate) url: String,
    pub(crate) origin_text: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub(crate) struct Message {
    pub(crate) role: Role,
    pub(crate) message: String,
    pub(crate) source_map: HashMap<i64, Source>,
}
#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub(crate) enum Role {
    User,
    Assistant,
    SourceMap,
    System,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub(crate) struct PartialMessage {
    pub(crate) role: Role,
    pub(crate) message: Option<String>,
    pub(crate) source_map: Option<HashMap<i64, Source>>,
    pub(crate) finished: Option<String>,
}

impl PartialMessage {
    pub(crate) fn done() -> Self {
        Self {
            message: None,
            source_map: None,
            finished: Some(String::from("DONE")),
            role: Role::Assistant,
        }
    }

    pub(crate) fn source(source: HashMap<i64, Source>) -> Self {
        Self {
            message: None,
            source_map: Some(source),
            finished: None,
            role: Role::SourceMap,
        }
    }

    pub(crate) fn content(content: String) -> Self {
        Self {
            message: Some(content),
            source_map: None,
            finished: None,
            role: Role::Assistant,
        }
    }

    pub(crate) fn message(self) -> Bytes {
        let message_string = &serde_json::to_string(&self).unwrap();

        Bytes::from(["event: message\ndata: ", message_string, "\n\n"].concat())
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub(crate) struct Conversation {
    pub(crate) messages: Vec<Message>,
}

pub(crate) trait CountSources {
    fn sources_count(&self) -> usize;
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub(crate) struct Query {
    pub(crate) message: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub(crate) struct Answer {
    pub(crate) message: String,
}
