mod docstore;
mod embed;
mod index;
mod protocol;
mod server;

use docstore::SqliteDocstore;
use server::run_server;

use crate::{embed::EmbedService, index::Index};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let index_path = "/home/michael/Development/wikirip/safe_space/index.faiss";
    let docstore_path = "/home/michael/Development/wikirip/safe_space/docstore.sqlitedb";

    let embedder = EmbedService::new(&"http://localhost:9000/embed")?;
    let docstore = SqliteDocstore::new(&docstore_path)
        .await
        .map_err(anyhow::Error::from)?;
    let index = Index::new(&index_path).map_err(anyhow::Error::from)?;
    let server = run_server(index, embedder, docstore)?;
    server.await.map_err(anyhow::Error::from)
}
