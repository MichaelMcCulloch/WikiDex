use std::sync::Arc;

use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::engine::IndexEngine;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = embedding_schema_example)]
pub(crate) struct Query(pub(crate) Vec<f32>, pub(crate) usize);

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = neighbors_schema_example)]
pub(crate) struct Neighbors(pub(crate) Vec<i64>);

#[derive(OpenApi)]
#[openapi(paths(query), components(schemas(Neighbors), schemas(Query),))]
pub(crate) struct ApiDoc;

#[utoipa::path(
    request_body = Embedding,
    responses(
        (status = 200, description = "Indices of neighbors", body = Neighbors)
    )
)]
#[post("/query")]
async fn query(
    Json(Query(embedding, neighbors)): Json<Query>,
    index: Data<Arc<IndexEngine>>,
) -> impl Responder {
    match index.query(embedding, neighbors).await {
        Ok(x) => HttpResponse::Ok().json(Neighbors(x)),
        Err(_e) => HttpResponse::InternalServerError().into(),
    }
}

fn embedding_schema_example() -> Query {
    Query(vec![0f32, 0.0, 0.0, 0.0], 4)
}
fn neighbors_schema_example() -> Neighbors {
    Neighbors(vec![0i64, 0, 0, 0])
}
