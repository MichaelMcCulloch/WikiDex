use std::error::Error;

pub(crate) trait SearchService {
    type E: Error;
    fn search(&mut self, query: &Vec<f32>, neighbors: usize) -> Result<Vec<i64>, Self::E>;
    fn batch_search(
        &mut self,
        query: &Vec<Vec<f32>>,
        neighbors: usize,
    ) -> Result<Vec<Vec<i64>>, Self::E>;
}
