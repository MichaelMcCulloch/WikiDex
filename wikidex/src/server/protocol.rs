use std::collections::HashMap;

use bytes::Bytes;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::formatter::{Cite, Provenance};

// type Source = (String, String, String, String);
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[schema(example = assistant_message_schema_example)]
pub(crate) struct Source {
    pub(crate) index: i64,
    pub(crate) citation: String,
    pub(crate) url: String,
    pub(crate) origin_text: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = assistant_message_schema_example)]
pub(crate) enum Message {
    User(String),
    Assistant(String),
    SourceMap(HashMap<i64, Source>),
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = assistant_partial_message_schema_example)]
pub(crate) struct PartialMessage {
    pub(crate) content: Option<String>,
    pub(crate) source_map: Option<HashMap<i64, Source>>,
    pub(crate) finished: Option<String>,
}

impl PartialMessage {
    pub(crate) fn done() -> Self {
        Self {
            content: None,
            source_map: None,
            finished: Some(String::from("DONE")),
        }
    }

    pub(crate) fn source(source: HashMap<i64, Source>) -> Self {
        Self {
            content: None,
            source_map: Some(source),
            finished: None,
        }
    }

    pub(crate) fn content(content: String) -> Self {
        Self {
            content: Some(content),
            source_map: None,
            finished: None,
        }
    }

    pub(crate) fn message(self) -> Bytes {
        let message_string = &serde_json::to_string(&self).unwrap();

        Bytes::from(["event: message\ndata: ", message_string, "\n\n"].concat())
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = conversation_schema_example)]
pub(crate) struct Conversation {
    pub(crate) messages: Vec<Message>,
}

pub(crate) trait CountSources {
    fn sources_count(&self) -> usize;
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = query_schema_example)]
pub(crate) struct Query {
    pub(crate) message: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = answer_schema_example)]
pub(crate) struct Answer {
    pub(crate) message: String,
}

fn assistant_message_schema_example() -> Message {
    Message::Assistant(String::from("String"))
}

fn assistant_partial_message_schema_example() -> PartialMessage {
    PartialMessage {
        content: Some(String::from(" fragment")),
        source_map: Some(source_map_example()),
        finished: Some(String::new()),
    }
}

fn source_map_example() -> HashMap<i64, Source> {
    let source = source_schema_example();
    let mut source_map = HashMap::new();
    let _ = source_map.insert(source.index, source);
    source_map
}
fn source_schema_example() -> Source {
    let p = Provenance::Wikipedia(
        "Bogonam-Foulbé".to_string(),
        DateTime::from_timestamp_millis(0).unwrap().date_naive(),
        DateTime::from_timestamp_millis(0).unwrap().date_naive(),
    );
    p.format(&crate::formatter::CitationStyle::Mla);
    Source {
        index: 987087,
        citation: p.format(&crate::formatter::CitationStyle::Mla),
        url: p.url(),
        origin_text: p.title(),
    }
}

fn user_message_schema_example() -> Message {
    Message::User(String::from("String"))
}
fn source_map_message_schema_example() -> Message {
    Message::SourceMap(source_map_example())
}
fn query_schema_example() -> Query {
    Query {
        message: String::from("String"),
    }
}
fn answer_schema_example() -> Answer {
    Answer {
        message: String::from("String"),
    }
}
fn conversation_schema_example() -> Conversation {
    Conversation {
        messages: vec![
            user_message_schema_example(),
            assistant_message_schema_example(),
        ],
    }
}
