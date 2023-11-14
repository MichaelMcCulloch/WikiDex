#[async_trait::async_trait]
pub(crate) trait DocumentService {
    type E: std::error::Error;
    type R;
    async fn retreive_batch(&self, indices: &Vec<Vec<i64>>) -> Result<Vec<Self::R>, Self::E>;
    async fn retreive(&self, indices: &Vec<i64>) -> Result<Self::R, Self::E>;
}
