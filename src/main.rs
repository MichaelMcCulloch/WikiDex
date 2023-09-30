extern crate intel_mkl_src;

mod embed;
mod index;
mod docstore;
mod server;

use anyhow::Result;
use docstore::{Docstore, SqliteDocstore}; 

use crate::{
    embed::{BertEmbed, Embed},
    index::{Index, Search},
};
#[actix_web::main]
async fn main() {

    std::env::set_var(
        "RUST_LOG",
        format!(
            r###"
                oracle=info,
            "###,
        ),
    );
    env_logger::init();
    let prompt = "What are the primary paradigms in Artificial Intelligence?";
    let embedder_path = "models/embed/thenlper/gte-small/";
    let index_path = "/home/michael/Development/retreival_augmented_generation/db/wikipedia.faiss";
    let docstore_path = "/home/michael/Development/retreival_augmented_generation/db/docstore.sqlite3";

    let embedder = BertEmbed::new(&embedder_path).unwrap();
    let index = Index::new(&index_path).unwrap();
    let docstore = SqliteDocstore::new(&docstore_path).await.unwrap();

    //// Embedder Stuff

    let embedding = embedder.embed(prompt).unwrap();
    let qquery = vec![embedding.clone(); 15];
    let result = index.search(&qquery, 4).unwrap();
    let documents = docstore.retreive(&result).await.unwrap();

}

