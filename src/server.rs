use std::sync::Arc;

use actix_web::web::Json;
use actix_web::{dev::Server, middleware, web::Data, HttpServer};
use actix_web::{post, App, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
enum Message {
    User(String),
    Assistant(String, Vec<String>),
}

#[derive(Debug, Deserialize, Serialize)]
struct Conversation(Vec<Message>);

use crate::docstore::Docstore;
use crate::embed::Embed;
use crate::index::Search;
use crate::{docstore::SqliteDocstore, embed::BertEmbed, index::Index};
#[post("/query")]
async fn query(
    Json(question): Json<String>,
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
    HttpResponse::Ok().json(response)
}

#[post("/conversation")]
async fn conversation(
    Json(conversation): Json<Conversation>,
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

    HttpResponse::Ok().json(conversation)
}

pub fn run_server(index: Index, embed: BertEmbed, docstore: SqliteDocstore) -> Result<Server> {
    let index = Arc::new(index);
    let embed = Arc::new(embed);
    let docstore = Arc::new(docstore);

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(Data::new(index.clone()))
            .app_data(Data::new(embed.clone()))
            .app_data(Data::new(docstore.clone()))
            .service(query)
    });
    server = server.bind("0.0.0.0:5000")?;
    let s = server.run();
    Ok(s)
}
