use async_openai::{config::OpenAIConfig, types::CreateEmbeddingRequestArgs, Client};

use super::error::EmbeddingServiceError;

pub(crate) struct EmbeddingClient {
    embedding_client: Client<OpenAIConfig>,
    embedding_model_name: String,
}

impl EmbeddingClient {
    pub(super) fn new(
        embedding_client: Client<OpenAIConfig>,
        embedding_model_name: String,
    ) -> Self {
        EmbeddingClient {
            embedding_client,
            embedding_model_name,
        }
    }
}

impl EmbeddingClient {
    pub(crate) async fn embed(&self, query: &str) -> Result<Vec<f32>, EmbeddingServiceError> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.embedding_model_name)
            .input([query])
            .build()
            .map_err(EmbeddingServiceError::AsyncOpenAiError)?;

        let response = self
            .embedding_client
            .embeddings()
            .create(request)
            .await
            .map_err(EmbeddingServiceError::AsyncOpenAiError)?;

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
