use super::{Answer, Conversation, Message, PartialMessage, Query, Role, Source};
use crate::inference::{Engine, QueryEngineError};
use axum::{
    extract::{
        ws::{Message as WebSocketMessage, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    Json,
};
use hyper::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(conversation, streaming_conversation),
    components(
        schemas(Message),
        schemas(Source),
        schemas(PartialMessage),
        schemas(Conversation),
        schemas(Query),
        schemas(Answer),
        schemas(Role)
    )
)]
pub(crate) struct ApiDoc;

#[utoipa::path(
    post,
    path = "/conversation",
    request_body(content = Conversation, content_type = "application/json"),
    responses(
        (status = 200, description = "AI Response", body = Message, content_type = "application/json"),
        (status = 204, description = "No user input"),
        (status = 400, description = "Empty Request")
    )
)]
pub(crate) async fn conversation(
    State(query_engine): State<Arc<Engine>>,
    Json(conversation): Json<Conversation>,
) -> impl IntoResponse {
    match query_engine
        .conversation(conversation, vec!["References:".to_string()])
        .await
    {
        Ok(message) => (StatusCode::OK, Json(message)).into_response(),
        Err(e) => {
            log::error!("{e}");
            match e {
                QueryEngineError::LastMessageIsNotUser | QueryEngineError::EmptyConversation => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Bad request"})),
                )
                    .into_response(),
                QueryEngineError::InvalidAgentResponse
                | QueryEngineError::LlmError(_)
                | QueryEngineError::IndexError(_)
                | QueryEngineError::DocstoreError(_)
                | QueryEngineError::EmbeddingServiceError(_)
                | QueryEngineError::Tera(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Internal server error"})),
                )
                    .into_response(),
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/streaming_conversation",
    request_body(content = Conversation, content_type = "application/json"),
    responses(
        (status = 200, description = "AI Response", body = PartialMessage, content_type = "text/event-stream"),
        (status = 204, description = "No user input"),
        (status = 400, description = "Empty Request")
    )
)]
pub(crate) async fn streaming_conversation(
    ws: WebSocketUpgrade,
    State(query_engine): State<Arc<Engine>>,
    Json(conversation_1): Json<Conversation>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, conversation_1, query_engine))
}

async fn handle_socket(
    mut socket: WebSocket,
    conversation: Conversation,
    query_engine: Arc<Engine>,
) {
    let (sender, receiver) = unbounded_channel();
    tokio::spawn(async move {
        if let Err(e) = query_engine
            .streaming_conversation(conversation, sender, vec!["References".to_string()])
            .await
        {
            log::error!("Streaming conversation error: {e}");
        }
    });

    let mut stream = UnboundedReceiverStream::new(receiver);
    while let Some(_message) = stream.next().await {
        if let Err(e) = socket
            .send(WebSocketMessage::Text("Hello".to_string()))
            .await
        {
            log::error!("WebSocket send error: {e}");
            break;
        }
    }
}
