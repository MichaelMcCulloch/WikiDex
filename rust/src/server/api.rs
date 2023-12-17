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
    paths(conversation, query, streaming_conversation),
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
    request_body = Query,
    responses(
        (status = 200, description = "Query Response", body = Answer)
    )
)]
#[post("/query")]
async fn query(
    Json(Query(question)): Json<Query>,
    query_engine: Data<Arc<Engine>>,
) -> impl Responder {
    match query_engine.query(&question).await {
        Ok(message) => HttpResponse::Ok().json(Answer(message)),
        Err(e) => {
            log::error!("{e}");
            match e {
                QueryEngineError::LastMessageIsNotUser | QueryEngineError::EmptyConversation => {
                    HttpResponse::BadRequest().into()
                }
                QueryEngineError::InvalidAgentResponse
                | QueryEngineError::UnableToLockIndex
                | QueryEngineError::LlmError(_)
                | QueryEngineError::IndexError(_)
                | QueryEngineError::DocstoreError(_)
                | QueryEngineError::EmbeddingServiceError(_) => {
                    HttpResponse::InternalServerError().into()
                }
            }
        }
    }
}
//request_body(content = Conversation, content_type = "application/json", example = json!([{"User":"What is the capital of France?"},{"Assistant":["The capital of france is Paris![0]",["https://en.wikipedia.org/wiki/France"]]},{"User":"And who is the current prime minister of france, and where were they born?"}])),

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
    match query_engine.conversation(conversation).await {
        Ok(message) => HttpResponse::Ok().json(message),
        Err(e) => {
            log::error!("{e}");
            match e {
                QueryEngineError::LastMessageIsNotUser | QueryEngineError::EmptyConversation => {
                    HttpResponse::BadRequest().into()
                }
                QueryEngineError::InvalidAgentResponse
                | QueryEngineError::UnableToLockIndex
                | QueryEngineError::LlmError(_)
                | QueryEngineError::IndexError(_)
                | QueryEngineError::DocstoreError(_)
                | QueryEngineError::EmbeddingServiceError(_) => {
                    HttpResponse::InternalServerError().into()
                }
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
    actix_web::rt::spawn(async move {
        let _ = query_engine
            .streaming_conversation(conversation_1, sender)
            .await
            .map_err(|e| log::error!("{e}"));
    });

    HttpResponse::Ok()
        .append_header(("content-type", "text/event-stream"))
        .append_header(("connection", "keep-alive"))
        .append_header(("cache-control", "no-cache"))
        .streaming(client)
}
