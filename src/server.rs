use std::path::Path;

use actix_web::{dev::Server, HttpServer};
use anyhow::Result;
use actix_web::App;


pub async fn run_server(_model_path: &Path, _index_path: &Path) -> Result<Server> {
    let mut server = HttpServer::new(move || App::new());
    server = server.bind("0.0.0.0:5000")?;
    let s = server.run();
    Ok(s)
}


