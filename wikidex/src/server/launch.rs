use super::{conversation, streaming_conversation, ApiDoc};

use crate::inference::Engine;
use axum::{
    routing::{post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

pub(crate) async fn run_server(
    engine: Engine,
    host: String,
    port: u16,
) -> Result<(), std::io::Error> {
    let engine = Arc::new(engine);

    let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any);
    // Create the router
    let app = Router::new()
        .route("/streaming_conversation", post(streaming_conversation))
        .route("/conversation", post(conversation))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/api-doc", ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .with_state(engine)
        .layer(cors);

    // Bind and serve
    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
