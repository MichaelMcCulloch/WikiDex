extern crate intel_mkl_src;

mod embed;
mod index;
mod docstore;
mod server;
mod protocol;

use docstore::SqliteDocstore;
use server::run_server;

use crate::{
    embed::BertEmbed,
    index::Index,
};



#[actix_web::main]
async fn main()-> anyhow::Result<()>  {

    std::env::set_var(
        "RUST_LOG",
        "info",
    ); 
    env_logger::init();
    let embedder_path = "models/embed/thenlper/gte-small/";
    let index_path = "db/thenlper/gte-small/index.faiss";
    let docstore_path = "db/thenlper/gte-small/docstore.sqlite3";

    let embedder = BertEmbed::new(&embedder_path)?;
    let docstore = SqliteDocstore::new(&docstore_path).await.map_err(anyhow::Error::from)?;
    let index = Index::new(&index_path).map_err(anyhow::Error::from)?;
    let server = run_server(index, embedder, docstore)?;
    server.await.map_err(anyhow::Error::from)

}

