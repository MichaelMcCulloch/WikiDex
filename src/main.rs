extern crate intel_mkl_src;

mod embed;
mod index;
mod server;

use anyhow::Result; 

use crate::{
    embed::{BertEmbed, Embed},
    index::{Index, Search},
};

fn main() {
    let prompt = "What are the primary paradigms in Artificial Intelligence?";
    let embedder_path = "models/embed/thenlper/gte-small/";
    let index_path = "/home/michael/Development/retreival_augmented_generation/db/wikipedia.faiss";

    let embedder = BertEmbed::new(&embedder_path).unwrap();
    let index = Index::new(&index_path).unwrap();

    //// Embedder Stuff

    let embedding = embedder.embed(prompt).unwrap();
    let qquery = vec![embedding.clone(); 15];
    let start = std::time::Instant::now();
    let result = index.search(&qquery, 4).unwrap();
    println!("Batch size {}, Search {:?}", 15, start.elapsed());
    for i in result.iter() {
        println!("{:?}", i);
    }
}

fn main2() -> Result<()> {
    std::env::set_var(
        "RUST_LOG",
        format!(
            r###"
                oracle=info,
            "###,
        ),
    );
    env_logger::init();
    Ok(())
}
