use std::sync::{Arc, Mutex};

use actix_cors::Cors;
use actix_web::{
    dev::Server,
    middleware, post,
    web::{Data, Json},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use crate::{
    docstore::Docstore,
    engine::{self, QueryEngineError},
    protocol::{llama::*, oracle::*},
};
use crate::{docstore::SqliteDocstore, embed::EmbedService, index::SearchIndex};
use crate::{embed::Embed, engine::Engine};
use crate::{engine::QueryEngine, index::Search};

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
    let response = query_engine.query(&question).await.unwrap();
    HttpResponse::Ok().json(Answer(response))
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
                | QueryEngineError::SerializationError
                | QueryEngineError::RequestError(_)
                | QueryEngineError::IndexError(_)
                | QueryEngineError::DocstoreError(_)
                | QueryEngineError::EmbeddingError(_) => HttpResponse::InternalServerError().into(),
            }
        }
    }
}

pub fn run_server(
    index: SearchIndex,
    embed: EmbedService,
    docstore: SqliteDocstore,
) -> Result<Server> {
    let openapi = ApiDoc::openapi();

    let index: Mutex<SearchIndex> = Mutex::new(index);
    let embed: EmbedService = embed;
    let docstore: SqliteDocstore = docstore;
    let engine = Arc::new(Engine::new(index, embed, docstore));

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            .app_data(Data::new(engine.clone()))
            .service(conversation)
            .service(query)
            .service(Redoc::with_url("/api-doc", openapi.clone()))
    });
    server = server.bind("0.0.0.0:5000")?;
    let s = server.run();
    Ok(s)
}
