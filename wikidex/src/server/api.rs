use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Responder,
};

use std::sync::Arc;
use utoipa::OpenApi;

use crate::{
    inference::{Engine, QueryEngineError},
    server::client::Client,
};

use super::{Answer, Conversation, Message, PartialMessage, Query, Source};

#[derive(OpenApi)]
#[openapi(
    paths(conversation, streaming_conversation),
    components(
        schemas(Message),
        schemas(Source),
        schemas(PartialMessage),
        schemas(Conversation),
        schemas(Query),
        schemas(Answer)
    )
)]
pub(crate) struct ApiDoc;

#[utoipa::path(
    request_body(content = Conversation, content_type = "application/json"),
    responses(
        (status = 200, description = "AI Response", body = Message, content_type = "application/json"),
        (status = 204, description = "No user input"),
        (status = 400, description = "Empty Request")
    )
)]
#[post("/conversation")]
async fn conversation(
    Json(conversation): Json<Conversation>,
    query_engine: Data<Arc<Engine>>,
) -> impl Responder {
    match query_engine
        .conversation(conversation, vec!["References:".to_string()])
        .await
    {
        Ok(message) => HttpResponse::Ok().json(message),
        Err(e) => {
            log::error!("{e}");
            match e {
                QueryEngineError::LastMessageIsNotUser | QueryEngineError::EmptyConversation => {
                    HttpResponse::BadRequest().into()
                }
                QueryEngineError::InvalidAgentResponse
                | QueryEngineError::LlmError(_)
                | QueryEngineError::IndexError(_)
                | QueryEngineError::DocstoreError(_)
                | QueryEngineError::EmbeddingServiceError(_)
                | QueryEngineError::Tera(_) => HttpResponse::InternalServerError().into(),
            }
        }
    }
}

#[utoipa::path(
    request_body(content = Conversation, content_type = "application/json"),
    responses(
        (status = 200, description = "AI Response", body = PartialMessage, content_type = "application/json"),
        (status = 204, description = "No user input"),
        (status = 400, description = "Empty Request")
    )
)]
#[post("/streaming_conversation")]
async fn streaming_conversation(
    Json(conversation_1): Json<Conversation>,
    query_engine: Data<Arc<Engine>>,
) -> impl Responder {
    let (client, sender) = Client::new();
    tokio::spawn(async move {
        let _ = query_engine
            .streaming_conversation(conversation_1, sender, vec!["References".to_string()])
            .await
            .map_err(|e| log::error!("{e}"));
    });

    HttpResponse::Ok()
        .append_header(("content-type", "text/event-stream"))
        .append_header(("connection", "keep-alive"))
        .append_header(("cache-control", "no-cache"))
        .streaming(client)
}
