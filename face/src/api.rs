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
pub(crate) struct Query {
    pub(crate) embedding: Vec<f32>,
    pub(crate) count: usize,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(example = neighbors_schema_example)]
pub(crate) struct Neighbors {
    pub neighbors: Vec<i64>,
}

#[derive(OpenApi)]
#[openapi(paths(query), components(schemas(Neighbors), schemas(Query),))]
pub(crate) struct ApiDoc;

#[utoipa::path(
    request_body = Query,
    responses(
        (status = 200, description = "Indices of neighbors", body = Neighbors)
    )
)]
#[post("/query")]
async fn query(
    Json(Query { embedding, count }): Json<Query>,
    index: Data<Arc<IndexEngine>>,
) -> impl Responder {
    match index.query(embedding, count).await {
        Ok(neighbors) => HttpResponse::Ok().json(Neighbors { neighbors }),
        Err(_e) => HttpResponse::InternalServerError().into(),
    }
}

fn embedding_schema_example() -> Query {
    Query {
        embedding: vec![0.1f32, 0.2, 0.3, 0.4],
        count: 4,
    }
}
fn neighbors_schema_example() -> Neighbors {
    Neighbors {
        neighbors: vec![0i64, 0, 0, 0],
    }
}
