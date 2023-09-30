extern crate intel_mkl_src;

mod server;
mod embed;
mod index;

use actix_web::rt;
use crossbeam::thread;
use anyhow::{Error as E, Result};
use candle_core::{
    backend::BackendDevice,  CudaDevice,  Device, Tensor,
};
use candle_nn::VarBuilder;
use candle_transformers::models::    bert::{BertModel, Config, DTYPE};
use faiss::ConcurrentIndex;
use tokenizers::{
    DecoderWrapper, ModelWrapper, NormalizerWrapper,  PostProcessorWrapper,
    PreTokenizerWrapper, Tokenizer,  TokenizerImpl,
};

use crate::{embed::{BertEmbed, Embed}, index::Index};

fn main() {
    let prompt = "What are the primary paradigms in Artificial Intelligence?";
    let embedder_path ="models/embed/thenlper/gte-small/";
    let index_path = "/home/michael/Development/retreival_augmented_generation/db/wikipedia.faiss";
    let device = candle_core::Device::Cuda(CudaDevice::new(0).unwrap());
    let embedder = BertEmbed::new(&embedder_path, &device).unwrap();
 
    let start = std::time::Instant::now();
    let index = Index::new(&index_path).unwrap();
    println!("Load Index {:?}", start.elapsed());

    //// Embedder Stuff

    // let embedding = embedder.embed(prompt).unwrap();
    // let qquery: Vec<f32> = vec![embedding.clone(); 15].into_iter().flatten().collect();
    // let start = std::time::Instant::now();
    // let result = index.search(&qquery, 4).unwrap();
    // println!("Batch size {}, Search {:?}", 15, start.elapsed());
    // for i in result.labels.iter() {
    //     continue;
    // }
 
}

fn main2() -> Result<()>{
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
