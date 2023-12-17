use faiss::{
    index::{pretransform::PreTransformIndexImpl, IndexImpl},
    Index,
};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use super::{IndexError, IndexSearchError, SearchService};

pub(crate) struct FaissIndex {
    index: PreTransformIndexImpl<IndexImpl>,
    dims: u32,
}

impl FaissIndex {
    pub(crate) fn new<P: AsRef<Path>>(index_path: &P) -> Result<Self, IndexError> {
        let start = Instant::now();

        let index_path = PathBuf::from(index_path.as_ref());
        if !index_path.exists() {
            return Err(IndexError::FileNotFound);
        }

        let index = faiss::read_index(index_path.to_str().expect("Index path is not a string"))
            .map_err(IndexError::IndexReadError)?
            .into_pre_transform()
            .map_err(IndexError::IndexFormatError)?;

        let dims = Index::d(&index);

        log::info!("Load Index {:?}", start.elapsed());
        Ok(FaissIndex { index, dims })
    }
}

impl SearchService for FaissIndex {
    type E = IndexSearchError;

    fn batch_search(
        &mut self,
        query: &Vec<Vec<f32>>,
        neighbors: usize,
    ) -> Result<Vec<Vec<i64>>, IndexSearchError> {
        let start = Instant::now();
        let flattened_query: Vec<f32> = query
            .iter()
            .all(|q| q.len() == self.dims as usize)
            .then(|| query.iter().flatten().copied().collect())
            .ok_or(IndexSearchError::IncorrectDimensions)?;

        let rs = self
            .index
            .search(&flattened_query, neighbors)
            .map_err(IndexSearchError::IndexSearchError)?;
        let x: Vec<i64> = rs.labels.iter().map(|i| i.to_native()).collect();
        let indices = x.chunks_exact(neighbors).map(|v| v.to_vec()).collect();
        log::debug!("Index {:?}", start.elapsed());
        Ok(indices)
    }

    fn search(&mut self, query: &Vec<f32>, neighbors: usize) -> Result<Vec<i64>, Self::E> {
        let start = Instant::now();
        let rs = self
            .index
            .search(query, neighbors)
            .map_err(IndexSearchError::IndexSearchError)?;
        let indices: Vec<i64> = rs.labels.iter().map(|i| i.to_native()).collect();
        log::debug!("Index {:?}", start.elapsed());
        Ok(indices)
    }
}
