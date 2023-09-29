use core::panic;
use std::{time::Duration, thread};

use faiss::{Index, index_factory, MetricType};
extern crate intel_mkl_src;

use candle_transformers::models::{bert::{BertModel, Config, DTYPE}, llama::{LlamaConfig, Llama, self}, quantized_llama::ModelWeights};

use anyhow::{anyhow, Error as E, Result};
use candle_core::{Tensor, CudaDevice,backend::BackendDevice, DType, Device, quantized::gguf_file};
use candle_nn::VarBuilder;
use hf_hub::{api::sync::Api, Cache, Repo, RepoType};
use tokenizers::{PaddingParams, Tokenizer};
const DEFAULT_SYSTEM_PROMPT: &str = "You are a helpful, respectful and honest assistant. Always answer as helpfully as possible, while being safe.  Your answers should not include any harmful, unethical, racist, sexist, toxic, dangerous, or illegal content. Please ensure that your responses are socially unbiased and positive in nature. If a question does not make any sense, or is not factually coherent, explain why instead of answering something not correct. If you don't know the answer to a question, please don't share false information.";
const B_STR: &str = "<s>";
const E_STR: &str = "</s>";
const B_INST: &str  = "[INST]";
const E_INST: &str = "[/INST]";
const B_SYS: &str = "<<SYS>>\n";
const E_SYS: &str = "\n<</SYS>>\n\n";

const SPECIAL_TAGS: [&str; 4] = [B_INST, E_INST, "<<SYS>>", "<</SYS>>"];
fn normalize(v: &Tensor) -> Result<Tensor> {
    Ok(v.broadcast_div(&v.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}


// <s>[INST] <<SYS>>\n{your_system_message}\n<</SYS>>\n\n{user_message_1} [/INST] {model_reply_1}</s><s>[INST] {user_message_2} [/INST]

fn prepare_llama2_prompt(system_prompt: &str, dialogue: &Vec<&str>) -> String {
    if dialogue.is_empty() {
        unimplemented!("The dialogue must contain at least one peice of text.")
    }

    let mut full_prompt = String::new();
    full_prompt.push_str(B_STR);
    
    let user_prompt = dialogue[0];
    let peice = if system_prompt.len() > 0 {
        format!("{B_INST} {B_SYS}{system_prompt}{E_SYS}{user_prompt} {E_INST}")
    } else {
        format!("{B_INST} {user_prompt} {E_INST}")
    };
    full_prompt.push_str(&peice);


    for i in 1..dialogue.len() {
        let next = dialogue[i];
        let next = if i == dialogue.len() - 1 &&  i % 2 != 0 {
            format!(" {next}")
        } else if i % 2 == 0 {
            format!(" {next} {E_INST}")
        } else  {
            format!(" {next}</s><s>{B_INST}")
        };
        full_prompt.push_str(&next)
    }
    
    // let mut index = 1;
    // let is_system_prompt_available = system_prompt.len() > 0;

    // if is_system_prompt_available {
    //     full_prompt.push_str(&format!("{B_INST}{B_SYS}{system_prompt} {E_SYS}{} {E_INST}", dialogue[index]));
    //     index += 1;
    // }

    // while index < dialogue.len() {
    //     full_prompt.push_str(&format!("{B_INST}{} {E_INST} ",  dialogue[index]));
    //     index += 1;

    //     if index < dialogue.len() {
    //         full_prompt.push_str(&format!("{B_INST}{} {E_INST} ", dialogue[index]));
    //         index += 1;
    //     }
    //     print!("{}", full_prompt);

    // }

    // full_prompt.push_str(&format!("{} ", B_INST));

    full_prompt
}
fn load_embedder(device: &Device) -> anyhow::Result<(BertModel, Tokenizer)>{

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

    let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[embed_model], DTYPE, &device)? };
    let model = BertModel::load(vb, &config)?;
    Ok((model, tokenizer))
}


fn load_llm(device: &Device) -> anyhow::Result<(Llama, Tokenizer, llama::Cache)>{

    let (llm_config, llm_tokenizer, llm_model) = {
        (
            "models/llm/TheBloke/Speechless-Llama2-Hermes-Orca-Platypus-WizardLM-13B-AWQ/config.json",
            "models/llm/TheBloke/Speechless-Llama2-Hermes-Orca-Platypus-WizardLM-13B-AWQ/tokenizer.json",
            "models/llm/TheBloke/Speechless-Llama2-Hermes-Orca-Platypus-WizardLM-13B-AWQ/model.safetensors",
        )
    };

    let config = std::fs::read_to_string(llm_config)?;
    let config: LlamaConfig = serde_json::from_str(&config)?;
    let config = config.into_config(true);
    let tokenizer = Tokenizer::from_file(llm_tokenizer).map_err(E::msg)?;
    let mut file = std::fs::File::open(&llm_model)?;
    let cache = llama::Cache::new(true, DType::BF16, &config, &device)?;

    let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[llm_model], DTYPE, &device)? };
    let model = Llama::load(vb, &cache,&config)?;
    Ok((model, tokenizer, cache))
}

fn main() {
    //// Embedder Stuff 
    // let start = std::time::Instant::now();
    let device = candle_core::Device::Cuda(CudaDevice::new(0).unwrap());

    let (llama_model, mut llama_tokenizer, llama_cache) = load_llm(&device).unwrap();
    let eos_token_id = llama_tokenizer.token_to_id(E_STR);
    let llm_prompt = "The capital of france is ";

    // let input = prepare_llama2_prompt("your_system_message", &vec![llm_prompt, "model_reply_1", "user_message_2", "model_reply_2",]);
    let input = prepare_llama2_prompt(DEFAULT_SYSTEM_PROMPT, &vec![llm_prompt]);
    println!("{},{:?}", input, eos_token_id);

    let mut tokens = llama_tokenizer
        .encode(input, true)
        .map_err(E::msg)
        .unwrap()
        .get_ids()
        .to_vec();


    /*
    let (embed_model, mut embed_tokenizer) = load_embedder(&device).unwrap();
    let prompt = "This is a segment of text";

    let tokenizer = embed_tokenizer.with_padding(None)
    .with_truncation(None)
    .map_err(E::msg).unwrap();

    let tokens = tokenizer
    .encode(prompt, true)
    .map_err(E::msg).unwrap()
    .get_ids()
    .to_vec();

    let token_ids = Tensor::new(&tokens[..], &device).unwrap().unsqueeze(0).unwrap();
    let token_type_ids = token_ids.zeros_like().unwrap();
    println!("Loaded and encoded {:?}", start.elapsed());


    let start = std::time::Instant::now();
    let embeddings = embed_model.forward(&token_ids, &token_type_ids).unwrap();

    
    // MANDATORY: Apply some avg-pooling by taking the mean embedding value for all tokens (including padding) and L2 Normalize
    let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3().unwrap();
    let embeddings = (embeddings.sum(1).unwrap() / (n_tokens as f64)).unwrap();
    let emb  = normalize(&embeddings).unwrap();
    println!("Embed {:?}", start.elapsed());

    //// FAISS Stuff
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
   
 */



} 
