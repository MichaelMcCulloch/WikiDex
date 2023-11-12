pub(crate) mod engine;
pub(crate) mod protocol;
use crate::server::engine::{Engine, QueryEngine, QueryEngineError};

use self::protocol::*;

use actix_cors::Cors;
use actix_web::{
    dev::Server,
    middleware, post,
    web::{Data, Json},
    App, HttpResponse, HttpServer, Responder,
};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

#[derive(OpenApi)]
#[openapi(
    paths(conversation, query),
    components(
        schemas(Message),
        schemas(Conversation),
        schemas(Query),
        schemas(Answer)
    )
)]
struct ApiDoc;

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
                QueryEngineError::IndexOutOfRange
                | QueryEngineError::InvalidAgentResponse
                | QueryEngineError::UnableToLockIndex
                | QueryEngineError::LlmError(_)
                | QueryEngineError::IndexError(_)
                | QueryEngineError::DocstoreError(_)
                | QueryEngineError::EmbeddingError(_) => HttpResponse::InternalServerError().into(),
            }
        }
    }
}
//request_body(content = Conversation, content_type = "application/json", example = json!([{"User":"What is the capital of France?"},{"Assistant":["The capital of france is Paris![0]",["https://en.wikipedia.org/wiki/France"]]},{"User":"And who is the current prime minister of france, and where were they born?"}])),

#[utoipa::path(
    request_body(content = Conversation, content_type = "application/json"),
    responses(
        (status = 200, description = "AI Response", body = Conversation, content_type = "application/json"),
        (status = 204, description = "No user input"),
        (status = 400, description = "Empty Request")
    )
)]
#[post("/conversation")]
async fn conversation(
    Json(mut conversation): Json<Conversation>,
    query_engine: Data<Arc<Engine>>,
) -> impl Responder {
    match query_engine.conversation(&conversation).await {
        Ok(message) => {
            conversation.0.push(message);
            HttpResponse::Ok().json(conversation)
        }
        Err(e) => {
            log::error!("{e}");
            match e {
                QueryEngineError::LastMessageIsNotUser | QueryEngineError::EmptyConversation => {
                    HttpResponse::BadRequest().into()
                }
                QueryEngineError::IndexOutOfRange
                | QueryEngineError::InvalidAgentResponse
                | QueryEngineError::UnableToLockIndex
                | QueryEngineError::LlmError(_)
                | QueryEngineError::IndexError(_)
                | QueryEngineError::DocstoreError(_)
                | QueryEngineError::EmbeddingError(_) => HttpResponse::InternalServerError().into(),
            }
        }
    }
}

pub(crate) fn run_server<S: AsRef<str>>(
    engine: Engine,
    host: S,
    port: u16,
) -> Result<Server, std::io::Error> {
    let openapi = ApiDoc::openapi();

    let engine = Arc::new(engine);

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            .app_data(Data::new(engine.clone()))
            .service(conversation)
            .service(query)
            .service(Redoc::with_url("/api-doc", openapi.clone()))
    });

    server = server.bind((host.as_ref(), port))?;
    let s = server.run();
    Ok(s)
}
