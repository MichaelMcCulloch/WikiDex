use std::error::Error;

use super::super::helper::wiki::DescribedTable;
#[async_trait::async_trait]
pub(crate) trait Process {
    type E: Error;
    async fn process(&self, markup: &str) -> Result<(String, Vec<DescribedTable>), Self::E>;
}
