use async_openai::{config::OpenAIConfig, types::CreateEmbeddingRequestArgs, Client};

use super::{error::EmbeddingServiceError, service::EmbedService};

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

#[async_trait::async_trait]
impl EmbedService for EmbeddingClient {
    type E = EmbeddingServiceError;
    async fn embed(&self, query: &str) -> Result<Vec<f32>, Self::E> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.embedding_model_name)
            .input([query])
            .build()
            .map_err(|e| EmbeddingServiceError::AsyncOpenAiError(e))?;

        let response = self
            .embedding_client
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
