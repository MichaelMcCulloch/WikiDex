use async_openai::{config::OpenAIConfig, types::CreateEmbeddingRequestArgs, Client};
use url::Url;

use crate::embed::{EmbedService, EmbeddingServiceError};

pub(crate) struct OpenAiEmbeddingService {
    client: Client<OpenAIConfig>,
    model_name: String,
}

impl OpenAiEmbeddingService {
    pub(crate) fn new<S: AsRef<str>>(openai_key: Option<String>, host: Url, model_name: S) -> Self {
        let openai_config = match openai_key {
            Some(key) => OpenAIConfig::new().with_api_key(key),
            None => OpenAIConfig::new().with_api_base(host),
        };

        let client = Client::with_config(openai_config);
        let model_name = model_name.as_ref().to_string();
        Self { client, model_name }
    }
}

#[async_trait::async_trait]
impl EmbedService for OpenAiEmbeddingService {
    type E = EmbeddingServiceError;
    async fn embed_batch(&self, queries: &[&str]) -> Result<Vec<Vec<f32>>, Self::E> {
        todo!()
    }
    async fn embed(&self, query: &str) -> Result<Vec<f32>, Self::E> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model_name)
            .input([query])
            .build()
            .map_err(|e| EmbeddingServiceError::AsyncOpenAiError(e))?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .map_err(|e| EmbeddingServiceError::AsyncOpenAiError(e))?;

        if response.data.len() > 1 {
            Err(EmbeddingServiceError::EmbeddingSizeMismatch(
                1,
                response.data.len(),
            ))
        } else if let Some(embedding) = response.data.into_iter().next() {
            Ok(embedding.embedding)
        } else {
            Err(EmbeddingServiceError::EmbeddingSizeMismatch(1, 0))
        }
    }
}
