use std::{sync::Arc};

use actix_web::{web::{Json, Data}, dev::Server, middleware, HttpServer, post, App, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, OpenApi};
use utoipa_redoc::{Redoc, Servable};

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


#[derive(Serialize, Deserialize, Clone, Debug)]
struct LlmInput {
    pub system: String,
    pub conversation: Vec<LlmMessage>
}


#[derive(Serialize, Deserialize, Clone, Debug)]
struct LlmMessage {
    pub role: String,
    pub message: String,
}

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
)]
struct ApiDoc;


fn format_document(index: i64, document: &String) -> String {
    format!("BEGIN DOCUMENT {index}\n§§§\n{document}\n§§§\nEND DOCUMENT {index}")
}

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
    let result = index.search(&embedding, 4).unwrap();
    let documents = docstore.retreive(&result).await.unwrap();
    let response = documents.iter().map(|(index, document)| format_document(*index, document)).collect::<Vec<String>>().join("\n\n");
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
    Json(Conversation(mut conversation)): Json<Conversation>,
    index: Data<Arc<Index>>,
    embed: Data<Arc<BertEmbed>>,
    docstore: Data<Arc<SqliteDocstore>>,
) -> impl Responder {
    log::info!("Conversation Received");
    let url = "http://0.0.0.0:5050/conversation";

    match conversation.last(){
        Some(Message::User(user_query)) => {
            let embedding = embed.embed(&user_query).unwrap();
            let result = index.search(&embedding, 4).unwrap();
            let documents = docstore.retreive(&result).await.unwrap();
            let formatted_document_list = documents.iter().map(|(index, document)|  format_document(*index, document)).collect::<Vec<String>>().join("\n\n");
            let input = LlmInput{
                system: format!(r###"You are a helpful, respectful and honest assistant. Always answer as helpfully as possible, while being safe.  Your answers should not include any harmful, unethical, racist, sexist, toxic, dangerous, or illegal content. Please ensure that your responses are socially unbiased and positive in nature. If a question does not make any sense, or is not factually coherent, explain why instead of answering something not correct. If you don't know the answer to a question, please don't share false information.

                Here are the provided documents:
                {formatted_document_list}

                Now, please provide a detailed answer to the query "{user_query}" using only the information from the provided documents. Make sure to cite the document numbers in your response, by putting the index of the document in square brackets after the statement. So if you were presented documents alpha, beta, gamma, delta in that order, if you cited delta you would put [3] after the statement, if you cited alpha, then [0] after the statement."###),
                conversation: vec![LlmMessage{role: String::from("user"), message: format!("{}", user_query)}]
            };
            let request_body = serde_json::to_string(&input).unwrap();
            let LlmInput {
                system,
                conversation: con
            } = reqwest::Client::new()
                .post(url)
                .json(&request_body)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            
            if let Some(LlmMessage { role, message }) = con.last() {
                if role == "assistant" {
                    conversation.push(Message::Assistant(message.to_string(), documents.iter().map(|(i, d)| format!("{i}")).collect()))
                } else {
                    return HttpResponse::InternalServerError().into()
                }
            } else {
                return HttpResponse::InternalServerError().into()
            }
      
            HttpResponse::Ok().json(Conversation(conversation))
        },
        Some(Message::Assistant(_, _)) => HttpResponse::NoContent().into(),
        None => HttpResponse::BadRequest().into(),
    }

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
