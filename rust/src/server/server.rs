use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{dev::Server, middleware, web::Data, App, HttpServer};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use crate::inference::Engine;

use super::{conversation, query, ApiDoc};

pub(crate) fn run_server<S: AsRef<str>>(
    engine: Engine,
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
            .service(conversation)
            .service(query)
            .service(Redoc::with_url("/api-doc", openapi.clone()))
    });

    server = server.bind((host.as_ref(), port))?;
    let s = server.run();
    Ok(s)
}