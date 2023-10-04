use std::{
    fmt::{self, Display, Formatter, Debug},
    path::Path,
};

use candle_core::{Tensor, CudaDevice, backend::BackendDevice};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use tokenizers::Tokenizer;

pub struct BertEmbed {
    embed_model: BertModel,
    embed_tokenizer: Tokenizer,
}

pub enum BertFile {
    Config,
    Tokenizer,
    Model,
    Directory,
}

pub enum BertLoadError {
    FileNotFound(BertFile),
    ConfigRead,
    ConfigParse,
    TokenizerRead,
    ModelLoad,
    ModelRead,
}

impl std::error::Error for BertLoadError{}
pub enum BertEmbedError {
    Tokenize,
    Embed,
    Tensor,
}

impl Display for BertFile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            BertFile::Config => write!(f, "Config"),
            BertFile::Tokenizer => write!(f, "Tokenizer"),
            BertFile::Model => write!(f, "Model"),
            BertFile::Directory => write!(f, "Directory"),
        }
    }
}

impl Display for BertLoadError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BertLoadError::FileNotFound(file) => {
                write!(f, "File not found: {}", file)
            }
            BertLoadError::ConfigRead => {
                write!(f, "Error reading configuration file")
            }
            BertLoadError::ConfigParse => {
                write!(f, "Error parsing configuration")
            }
            BertLoadError::TokenizerRead => {
                write!(f, "Error reading tokenizer file")
            }
            BertLoadError::ModelLoad => {
                write!(f, "Error loading model")
            }
            BertLoadError::ModelRead => {
                write!(f, "Error reading model file")
            }
        }
    }
}
impl Display for BertEmbedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BertEmbedError::Tokenize => write!(f, "Error during tokenization"),
            BertEmbedError::Embed => write!(f, "Error during embedding"),
            BertEmbedError::Tensor => write!(f, "Error creating tensor"),
        }
    }
}

impl Debug for BertFile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Debug for BertLoadError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}


impl Debug for BertEmbedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl BertEmbed {
    pub fn new<P: AsRef<Path>>(model_path: &P) -> Result<Self, BertLoadError> {
        let model_path = model_path.as_ref();
        let device = candle_core::Device::Cuda(CudaDevice::new(0).unwrap());

        let start = std::time::Instant::now();

        if !model_path.exists() {
            return Err(BertLoadError::FileNotFound(BertFile::Directory));
        }
        let (embed_config, embed_tokenizer, embed_model) = {
            (
                {
                    let config_path = model_path.join("config.json");
                    config_path
                        .exists()
                        .then(|| config_path)
                        .ok_or(BertLoadError::FileNotFound(BertFile::Config))?
                },
                {
                    let tokenizer_path = model_path.join("tokenizer.json");
                    tokenizer_path
                        .exists()
                        .then(|| tokenizer_path)
                        .ok_or(BertLoadError::FileNotFound(BertFile::Tokenizer))?
                },
                {
                    let model_path = model_path.join("model.safetensors");
                    model_path
                        .exists()
                        .then(|| model_path)
                        .ok_or(BertLoadError::FileNotFound(BertFile::Model))?
                },
            )
        };

        let config =
            std::fs::read_to_string(embed_config).map_err(|_| BertLoadError::ConfigRead)?;
        let config: Config =
            serde_json::from_str(&config).map_err(|_| BertLoadError::ConfigParse)?;

        let mut tokenizer =
            Tokenizer::from_file(embed_tokenizer).map_err(|_| BertLoadError::TokenizerRead)?;
        let tokenizer_impl = tokenizer
            .with_padding(None)
            .with_truncation(None)
            .map_err(|_| BertLoadError::TokenizerRead)?
            .to_owned();

        let embed_tokenizer = Tokenizer::from(tokenizer_impl);
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[embed_model], DTYPE, &device)
                .map_err(|_| BertLoadError::ModelRead)?
        };
        let embed_model = BertModel::load(vb, &config).map_err(|_| BertLoadError::ModelLoad)?;
        log::info!("Load Embedder {:?}", start.elapsed());
        Ok(BertEmbed {
            embed_model,
            embed_tokenizer,
        })
    }
}

pub trait Embed {
    type E;
    fn embed(&self, str: &str) -> Result<Vec<f32>, Self::E>;
}

impl Embed for BertEmbed {
    type E = BertEmbedError;
    fn embed(&self, query: &str) -> Result<Vec<f32>, BertEmbedError> {
        let start = std::time::Instant::now();
        let tokens = self
            .embed_tokenizer
            .encode(query, true)
            .map_err(|_| BertEmbedError::Tokenize)?
            .get_ids()
            .to_vec();

        let token_ids = Tensor::new(&tokens[..], &self.embed_model.device)
            .map_err(|_| BertEmbedError::Tensor)?
            .unsqueeze(0)
            .map_err(|_| BertEmbedError::Tensor)?;
        let token_type_ids = token_ids.zeros_like().map_err(|_| BertEmbedError::Tensor)?;

        let embeddings = &self
            .embed_model
            .forward(&token_ids, &token_type_ids)
            .map_err(|_| BertEmbedError::Embed)?;

        // MANDATORY: Apply some avg-pooling by taking the mean embedding value for all tokens (including padding) and L2 Normalize
        let (_n_sentence, n_tokens, _hidden_size) =
            embeddings.dims3().map_err(|_| BertEmbedError::Tensor)?;
        let embeddings = (embeddings.sum(1).map_err(|_| BertEmbedError::Tensor)?
            / (n_tokens as f64))
            .map_err(|_| BertEmbedError::Tensor)?;
        let emb = normalize(&embeddings).map_err(|_| BertEmbedError::Tensor)?;
        let e: Vec<f32> = emb
            .get(0)
            .map_err(|_| BertEmbedError::Tensor)?
            .to_vec1()
            .map_err(|_| BertEmbedError::Tensor)?;
        log::debug!("Embed {:?}", start.elapsed());
        Ok(e)
    }
}

fn normalize(v: &Tensor) -> anyhow::Result<Tensor> {
    Ok(v.broadcast_div(&v.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}
