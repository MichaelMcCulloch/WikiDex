use bytes::Bytes;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// type Source = (String, String, String, String);
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[schema(example = assistant_message_schema_example)]
pub(crate) struct Source {
    pub(crate) ordinal: usize,
    pub(crate) index: i64,
    pub(crate) citation: String,
    pub(crate) url: String,
    pub(crate) origin_text: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = assistant_message_schema_example)]
pub(crate) enum Message {
    User(String),
    Assistant(String, Vec<Source>),
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = assistant_partial_message_schema_example)]
pub(crate) struct PartialMessage {
    pub(crate) content: Option<String>,
    pub(crate) source: Option<Source>,
    pub(crate) finished: Option<String>,
}

impl PartialMessage {
    pub(crate) fn done() -> Self {
        Self {
            content: None,
            source: None,
            finished: Some(String::from("DONE")),
        }
    }

    pub(crate) fn source(source: Source) -> Self {
        Self {
            content: None,
            source: Some(source),
            finished: None,
        }
    }

    pub(crate) fn content(content: String) -> Self {
        Self {
            content: Some(content),
            source: None,
            finished: None,
        }
    }
    pub(crate) fn content_and_source(content: String, source: Source) -> Self {
        Self {
            content: Some(content),
            source: Some(source),
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
impl CountSources for Vec<Message> {
    fn sources_count(&self) -> usize {
        self.iter().fold(0usize, |acc, message| match message {
            Message::User(_) => acc,
            Message::Assistant(_, s) => s.len() + acc,
        })
    }
}
impl CountSources for Conversation {
    fn sources_count(&self) -> usize {
        self.messages.sources_count()
    }
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
    Message::Assistant(
        String::from("String"),
        vec![
            source_schema_example(),
            source_schema_example(),
            source_schema_example(),
            source_schema_example(),
        ],
    )
}

fn assistant_partial_message_schema_example() -> PartialMessage {
    PartialMessage {
        content: Some(String::from(" fragment")),
        source: Some(source_schema_example()),
        finished: Some(String::new()),
    }
}

fn source_schema_example() -> Source {
    Source { ordinal: 0, index: 987087, citation: "Bogonam-Foulbé. 2023, December 1. In Wikipedia. Retrieved December 1, 2023, from https://en.wikipedia.org/wiki/Bogonam-Foulbé".to_string(), url: "https://en.wikipedia.org/wiki/Bogonam-Foulbé".to_string(), origin_text: "Bogonam-Foulbé is a village in the Kongoussi Department of Bam Province in northern Burkina Faso. It has a population of 205.".to_string() }
}

fn user_message_schema_example() -> Message {
    Message::User(String::from("String"))
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
