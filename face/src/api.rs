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
pub(crate) struct Embedding(pub(crate) Vec<f32>);

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = neighbors_schema_example)]
pub(crate) struct Neighbors(pub(crate) Vec<usize>);

#[derive(OpenApi)]
#[openapi(paths(query), components(schemas(Neighbors), schemas(Embedding),))]
pub(crate) struct ApiDoc;

#[utoipa::path(
    request_body = Embedding,
    responses(
        (status = 200, description = "Indices of neighbors", body = Neighbors)
    )
)]
#[post("/query")]
async fn query(
    Json(Embedding(_embedding)): Json<Embedding>,
    _index: Data<Arc<IndexEngine>>,
) -> impl Responder {
    HttpResponse::Ok().json(Neighbors(vec![0usize]))
}

fn embedding_schema_example() -> Embedding {
    Embedding(vec![0f32, 0.0, 0.0, 0.0])
}
fn neighbors_schema_example() -> Neighbors {
    Neighbors(vec![0usize, 0, 0, 0])
}
