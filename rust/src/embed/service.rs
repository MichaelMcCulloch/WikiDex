use std::error::Error;

#[async_trait::async_trait]
pub(crate) trait EmbedService {
    type E: Error;
    async fn embed(&self, str: &[&str]) -> Result<Vec<Vec<f32>>, Self::E>;
}
