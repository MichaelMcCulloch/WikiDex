extern crate intel_mkl_src;

mod server;

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

fn load_embedder(device: &Device) -> anyhow::Result<(BertModel, Tokenizer)> {
    let start = std::time::Instant::now();
    let (embed_config, embed_tokenizer, embed_model) = {
        (
            "models/embed/thenlper/gte-small/config.json",
            "models/embed/thenlper/gte-small/tokenizer.json",
            "models/embed/thenlper/gte-small/model.safetensors",
        )
    };

    let config = std::fs::read_to_string(embed_config)?;
    let config: Config = serde_json::from_str(&config)?;

    let tokenizer = Tokenizer::from_file(embed_tokenizer).map_err(E::msg)?;

    let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[embed_model], DTYPE, &device)? };
    let model = BertModel::load(vb, &config)?;
    println!("Load Embedder {:?}", start.elapsed());
    Ok((model, tokenizer))
}
fn normalize(v: &Tensor) -> Result<Tensor> {
    Ok(v.broadcast_div(&v.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}

fn embed_query(
    query: &str,
    model: &BertModel,
    tokenizer: &TokenizerImpl<
        ModelWrapper,
        NormalizerWrapper,
        PreTokenizerWrapper,
        PostProcessorWrapper,
        DecoderWrapper,
    >,
    device: &Device,
) -> Vec<f32> {
    let start = std::time::Instant::now();
    let tokens = tokenizer
        .encode(query, true)
        .map_err(E::msg)
        .unwrap()
        .get_ids()
        .to_vec();

    let token_ids = Tensor::new(&tokens[..], &device)
        .unwrap()
        .unsqueeze(0)
        .unwrap();
    let token_type_ids = token_ids.zeros_like().unwrap();

    let embeddings = model.forward(&token_ids, &token_type_ids).unwrap();

    // MANDATORY: Apply some avg-pooling by taking the mean embedding value for all tokens (including padding) and L2 Normalize
    let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3().unwrap();
    let embeddings = (embeddings.sum(1).unwrap() / (n_tokens as f64)).unwrap();
    let emb = normalize(&embeddings).unwrap();
    let e: Vec<f32> = emb.get(0).unwrap().to_vec1().unwrap();
    println!("Embed {:?}", start.elapsed());
    e
}

fn main2() {
    let device = candle_core::Device::Cuda(CudaDevice::new(0).unwrap());

    
    let start = std::time::Instant::now();
    
    let index = faiss::read_index(
        "/home/michael/Development/retreival_augmented_generation/db/wikipedia.faiss",
    )
    .unwrap().into_flat().unwrap();
    

    println!("Load Index {:?}", start.elapsed());

    //// Embedder Stuff

    let (embed_model, mut embed_tokenizer) = load_embedder(&device).unwrap();
    let prompt = "What are the primary paradigms in Artificial Intelligence?";

    let embed_tokenizer: &TokenizerImpl<_, _, _, _, _> = embed_tokenizer
        .with_padding(None)
        .with_truncation(None)
        .unwrap();

    let embedding = embed_query(prompt, &embed_model, &embed_tokenizer, &device);
    for i in 0..32 {
        let qquery: Vec<f32> = vec![embedding.clone(); 15].into_iter().flatten().collect();
        let start = std::time::Instant::now();
        let result = index.search(&qquery, 4).unwrap();
        println!("Batch size {}, Search {:?}", i, start.elapsed());
        for i in result.labels.iter() {
           continue;
        }
    }
 
}

fn main() -> Result<()>{
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
