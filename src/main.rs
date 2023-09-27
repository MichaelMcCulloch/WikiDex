use std::{time::Duration, thread};

use faiss::{Index, index_factory, MetricType};
extern crate intel_mkl_src;

use candle_transformers::models::bert::{BertModel, Config, DTYPE};

use anyhow::{anyhow, Error as E, Result};
use candle_core::{Tensor, CudaDevice,backend::BackendDevice};
use candle_nn::VarBuilder;
use hf_hub::{api::sync::Api, Cache, Repo, RepoType};
use tokenizers::{PaddingParams, Tokenizer};

fn normalize(v: &Tensor) -> Result<Tensor> {
    Ok(v.broadcast_div(&v.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}


fn load_embedder() -> anyhow::Result<(BertModel, Tokenizer)>{
    let device = candle_core::Device::Cuda(CudaDevice::new(0)?);

    let (embed_config, embed_tokenizer, embed_model) = {
        (
            "models/embed/thenlper/gte-small/config.json",
            "models/embed/thenlper/gte-small/tokenizer.json",
            "models/embed/thenlper/gte-small/model.safetensors"
        )
    };

    let config = std::fs::read_to_string(embed_config)?;
    let config: Config = serde_json::from_str(&config)?;
    let tokenizer = Tokenizer::from_file(embed_tokenizer).map_err(E::msg)?;

    let vb =
    unsafe { VarBuilder::from_mmaped_safetensors(&[embed_model], DTYPE, &device)? };
    let model = BertModel::load(vb, &config)?;
    Ok((model, tokenizer))
}

fn main() {

    let start = std::time::Instant::now();
    let (model, mut tokenizer) = load_embedder().unwrap();
    let device = &model.device;
    let prompt = "This is a segment of text";

    let tokenizer = tokenizer.with_padding(None)
    .with_truncation(None)
    .map_err(E::msg).unwrap();

    let tokens = tokenizer
    .encode(prompt, true)
    .map_err(E::msg).unwrap()
    .get_ids()
    .to_vec();

    let token_ids = Tensor::new(&tokens[..], device).unwrap().unsqueeze(0).unwrap();
    let token_type_ids = token_ids.zeros_like().unwrap();
    println!("Loaded and encoded {:?}", start.elapsed());


    let start = std::time::Instant::now();
    let embeddings = model.forward(&token_ids, &token_type_ids).unwrap();

    
    // MANDATORY: Apply some avg-pooling by taking the mean embedding value for all tokens (including padding) and L2 Normalize
    let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3().unwrap();
    let embeddings = (embeddings.sum(1).unwrap() / (n_tokens as f64)).unwrap();
    let emb  = normalize(&embeddings).unwrap();
    println!("Embed {:?}", start.elapsed());


    let mut index = index_factory(64, "Flat", MetricType::L2).unwrap();
    index.add(&[0f32;64]).unwrap();
    index.add(&[0f32;64]).unwrap();
    index.add(&[0f32;64]).unwrap();
    index.add(&[0f32;64]).unwrap();
    index.add(&[0f32;64]).unwrap();

    let start = std::time::Instant::now();
    let result = index.search(&[0f32;64], 5).unwrap();

    
    println!("Search {:?}", start.elapsed());
    for (i, (l, d)) in result.labels.iter()
        .zip(result.distances.iter())
        .enumerate()
    {
        println!("#{}: {} (D={})", i + 1, *l, *d);
    }
   




} 
