

pub(crate) mod oracle {

    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    #[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
    pub(crate) enum Message {
        User(String),
        Assistant(String, Vec<(String, String)>),
    }

    #[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
    pub(crate) struct Conversation(pub(crate) Vec<Message>);

    #[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
    pub(crate) struct Query(pub(crate) String);

    #[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
    pub(crate) struct Answer(pub(crate) String);
}

pub(crate) mod llama {

    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub(crate) struct LlmInput {
        pub(crate) system: String,
        pub(crate) conversation: Vec<LlmMessage>
    }


    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub(crate) struct LlmMessage {
        pub(crate) role: String,
        pub(crate) message: String,
    }

}