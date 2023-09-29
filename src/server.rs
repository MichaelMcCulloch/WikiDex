
use std::path::Path;

use anyhow::{Error as E, Result};
use actix_web::{dev::Server, HttpServer};


use actix_web::{rt, App};
use crossbeam::thread;

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
pub async fn run_server(
    model_path: &Path,
    index_path: &Path,
) -> Result<Server> {
    let mut server = HttpServer::new(move || {
        App::new()
            
    });
    server = server.bind("0.0.0.0:5000")?;
    let s = server.run();
    Ok(s)

}


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