use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{dev::Server, middleware, web::Data, App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::query;
use crate::api::ApiDoc;
use crate::engine::IndexEngine;

pub(crate) fn run_server<S: AsRef<str>>(
    engine: IndexEngine,
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
            .service(query)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
    });

    server = server.bind((host.as_ref(), port))?;
    let s = server.run();
    Ok(s)
}
