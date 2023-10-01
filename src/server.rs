use std::sync::Arc;

use actix_web::{web::{Json, Data}, dev::Server, middleware, HttpServer, post, App, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{ToSchema, OpenApi};
use utoipa_redoc::{Redoc, Servable};




/* [
  {
    "User": "string"
  },
  {
    "Assistant": [
      {},
      {}
    ]
  }
] */



#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
enum Message {
    User(String),
    Assistant(String, Vec<String>),
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
struct Conversation(Vec<Message>);

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]

struct Query(String);

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]

struct Answer(String);


use crate::docstore::Docstore;
use crate::embed::Embed;
use crate::index::Search;
use crate::{docstore::SqliteDocstore, embed::BertEmbed, index::Index};




#[derive(OpenApi)]
#[openapi(
    paths(
        conversation,
        query
    ),
    components(
        schemas(Message),
        schemas(Conversation),
        schemas(Query),
        schemas(Answer)
    ),
    // tags(
    //     (name = "todo", description = "Todo management endpoints.")
    // ),
    // modifiers(&SecurityAddon)
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
    index: Data<Arc<Index>>,
    embed: Data<Arc<BertEmbed>>,
    docstore: Data<Arc<SqliteDocstore>>,
) -> impl Responder {
    log::info!("Query Received");


    let embedding = embed.embed(&question).unwrap();
    let qquery = vec![embedding.clone(); 1];
    let result = index.search(&qquery, 4).unwrap();
    let documents = docstore.retreive(&result).await.unwrap();
    let response = documents.get(0).unwrap().join("|");
    HttpResponse::Ok().json(Answer(response))
}

#[utoipa::path(
    request_body(content = Conversation, content_type = "application/json", example = json!([{"User":"What is the capital of France?"},{"Assistant":["The capital of france is Paris![0]",["https://en.wikipedia.org/wiki/France"]]},{"User":"And who is the current prime minister of france, and where were they born?"}])),
    responses(
        (status = 200, description = "AI Response", body = Conversation, content_type = "application/json",example = json!([{"User":"What is the capital of France?"},{"Assistant":["The capital of france is Paris![0]",["https://en.wikipedia.org/wiki/France"]]},{"User":"And who is the current prime minister of france, and where were they born?"},{"Assistant":["The president of the French Republic as of 2023 is Emmanuel Macron![0] and he was born in Amiens, Somme, France[1]",["https://en.wikipedia.org/wiki/President_of_France","https://en.wikipedia.org/wiki/Emmanuel_Macron"]]}])),
        (status = 204, description = "No user input")

    )
)]
#[post("/conversation")]
async fn conversation(
    Json(Conversation(conversation)): Json<Conversation>,
    index: Data<Arc<Index>>,
    embed: Data<Arc<BertEmbed>>,
    docstore: Data<Arc<SqliteDocstore>>,
) -> impl Responder {
    log::info!("Conversation Received");

    let conversation = vec![
        Message::User(String::from("What is the capital of France?")),
        Message::Assistant(String::from("The capital of france is Paris![0]"), vec![String::from("https://en.wikipedia.org/wiki/France")]),
        Message::User(String::from("And who is the current prime minister of france, and where were they born?")),
        Message::Assistant(String::from("The president of the French Republic as of 2023 is Emmanuel Macron![0] and he was born in Amiens, Somme, France[1]"), vec![String::from("https://en.wikipedia.org/wiki/President_of_France"), String::from("https://en.wikipedia.org/wiki/Emmanuel_Macron")]),
    ];

    HttpResponse::Ok().json(Conversation(conversation))
}

pub fn run_server(index: Index, embed: BertEmbed, docstore: SqliteDocstore) -> Result<Server> {

    let openapi = ApiDoc::openapi();

    let index = Arc::new(index);
    let embed = Arc::new(embed);
    let docstore = Arc::new(docstore);

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(Data::new(index.clone()))
            .app_data(Data::new(embed.clone()))
            .app_data(Data::new(docstore.clone()))
            .service(conversation)
            .service(query)
            .service(Redoc::with_url("/api-doc", openapi.clone()))
    });
    server = server.bind("0.0.0.0:5000")?;
    let s = server.run();
    Ok(s)
}
