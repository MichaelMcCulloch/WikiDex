use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[schema(example = assistant_message_schema_example)]
pub(crate) enum Message {
    User(String),
    Assistant(String, Vec<(String, String)>),
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[schema(example = conversation_schema_example)]
pub(crate) struct Conversation(pub(crate) Vec<Message>);

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[schema(example = query_schema_example)]
pub(crate) struct Query(pub(crate) String);

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[schema(example = answer_schema_example)]
pub(crate) struct Answer(pub(crate) String);

fn assistant_message_schema_example() -> Message {
    Message::Assistant(
        String::from("String"),
        vec![
            (String::from("1"), String::from("Referenced Text 1")),
            (String::from("2"), String::from("Referenced Text 2")),
            (String::from("3"), String::from("Referenced Text 3")),
        ],
    )
}
fn user_message_schema_example() -> Message {
    Message::User(String::from("String"))
}
fn query_schema_example() -> Query {
    Query(String::from("String"))
}
fn answer_schema_example() -> Answer {
    Answer(String::from("String"))
}
fn conversation_schema_example() -> Conversation {
    Conversation(vec![
        user_message_schema_example(),
        assistant_message_schema_example(),
    ])
}
