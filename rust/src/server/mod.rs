pub(crate) mod protocol;

use actix_cors::Cors;
use actix_web::{
    dev::Server,
    middleware, post,
    web::{self, Data, Json},
    App, HttpResponse, HttpServer, Responder,
};
use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};
use url::Url;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use self::protocol::*;
use crate::engine::QueryEngine;
use crate::engine::QueryEngineError;
use crate::{config::EngineConfig, engine::Engine};

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

#[utoipa::path(
    request_body(content = Conversation, content_type = "application/json", example = json!([{"User":"What is the capital of France?"},{"Assistant":["The capital of france is Paris![0]",["https://en.wikipedia.org/wiki/France"]]},{"User":"And who is the current prime minister of france, and where were they born?"}])),
    responses(
        (status = 200, description = "AI Response", body = Conversation, content_type = "application/json",example = json!([{"User":"What is the capital of France?"},{"Assistant":["The capital of france is Paris![0]",["https://en.wikipedia.org/wiki/France"]]},{"User":"And who is the current prime minister of france, and where were they born?"},{"Assistant":["The president of the French Republic as of 2023 is Emmanuel Macron![0] and he was born in Amiens, Somme, France[1]",["https://en.wikipedia.org/wiki/President_of_France","https://en.wikipedia.org/wiki/Emmanuel_Macron"]]}])),
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

pub(crate) fn run_server(engine: Engine, config: EngineConfig) -> Result<Server, std::io::Error> {
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

    let url: Url = config.into();

    let host = url.host().expect("Host is not valid");
    let port = url.port().expect("Port is not valid");
    server = server.bind((host.to_string(), port))?;
    let s = server.run();
    Ok(s)
}
