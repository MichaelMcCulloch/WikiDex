use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{CreateEmbeddingRequestArgs, ListModelResponse},
    Client,
};

use super::{error::EmbeddingServiceError, EmbeddingClientService};

pub(crate) struct EmbeddingClient {
    embedding_client: Client<OpenAIConfig>,
    embedding_model_name: String,
}

impl EmbeddingClient {
    pub(crate) fn new(
        embedding_client: Client<OpenAIConfig>,
        embedding_model_name: String,
    ) -> Self {
        EmbeddingClient {
            embedding_client,
            embedding_model_name,
        }
    }
}

impl EmbeddingClientService for EmbeddingClient {
    async fn up(&self) -> Result<ListModelResponse, OpenAIError> {
        self.embedding_client.models().list().await
    }

    async fn embed_batch(
        &self,
        queries: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, EmbeddingServiceError> {
        let request: async_openai::types::CreateEmbeddingRequest =
            CreateEmbeddingRequestArgs::default()
                .model(&self.embedding_model_name)
                .input(&queries)
                .build()?;

        let response = self.embedding_client.embeddings().create(request).await?;

        if response.data.len() != queries.len() {
            Err(EmbeddingServiceError::EmbeddingSizeMismatch(
                queries.len(),
                response.data.len(),
            ))
        } else {
            Ok(response
                .data
                .into_iter()
                .map(|e| e.embedding)
                .collect::<Vec<_>>())
        }
    }
    async fn embed(&self, query: &str) -> Result<Vec<f32>, EmbeddingServiceError> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.embedding_model_name)
            .input([query])
            .build()?;

        let response = self.embedding_client.embeddings().create(request).await?;

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
