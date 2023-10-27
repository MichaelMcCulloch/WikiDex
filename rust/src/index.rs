use std::{
    fmt::{Debug, Display, Formatter},
    mem::{self, ManuallyDrop},
    path::{Path, PathBuf},
    ptr,
};

use faiss::{
    index::{
        flat::FlatIndexImpl,
        pretransform::{PreTransformIndex, PreTransformIndexImpl},
        FromInnerPtr, IndexImpl, NativeIndex, UpcastIndex,
    },
    index_factory,
    vector_transform::{PCAMatrix, PCAMatrixImpl, VectorTransform},
    ConcurrentIndex, FlatIndex, Index, MetricType,
};

pub struct FaissIndex {
    index: PreTransformIndexImpl<IndexImpl>,
    dims: u32,
}

impl FaissIndex {
    pub(crate) fn new<P: AsRef<Path>>(index_path: &P) -> Result<Self, IndexLoadError> {
        let start = std::time::Instant::now();

        let index_path = PathBuf::from(index_path.as_ref());
        if !index_path.exists() {
            return Err(IndexLoadError::FileNotFound);
        }

        let index = faiss::read_index(index_path.to_str().expect("Index path is not a string"))
            .map_err(|e| IndexLoadError::IndexReadError(e))?
            .into_pre_transform()
            .map_err(|e| IndexLoadError::IndexFormatError(e))?;

        let dims = faiss::Index::d(&index);

        log::info!("Load Index {:?}", start.elapsed());
        Ok(FaissIndex { index, dims })
    }
}

pub trait SearchService {
    type E;
    fn search(&mut self, query: &Vec<f32>, neighbors: usize) -> Result<Vec<i64>, Self::E>;
    fn batch_search(
        &mut self,
        query: &Vec<Vec<f32>>,
        neighbors: usize,
    ) -> Result<Vec<Vec<i64>>, Self::E>;
}

impl SearchService for FaissIndex {
    type E = IndexSearchError;

    fn batch_search(
        &mut self,
        query: &Vec<Vec<f32>>,
        neighbors: usize,
    ) -> Result<Vec<Vec<i64>>, IndexSearchError> {
        let start = std::time::Instant::now();
        let flattened_query: Vec<f32> = query
            .iter()
            .all(|q| q.len() == self.dims as usize)
            .then(|| query.into_iter().flatten().map(|f| *f).collect())
            .ok_or(IndexSearchError::IncorrectDimensions)?;

        let rs = self
            .index
            .search(&flattened_query, neighbors)
            .map_err(|d| IndexSearchError::IndexSearchError(d))?;
        let x: Vec<i64> = rs.labels.iter().map(|i| i.to_native()).collect();
        let indices = x.chunks_exact(neighbors).map(|v| v.to_vec()).collect();
        log::debug!("Index {:?}", start.elapsed());
        Ok(indices)
    }

    fn search(&mut self, query: &Vec<f32>, neighbors: usize) -> Result<Vec<i64>, Self::E> {
        let start = std::time::Instant::now();
        let rs = self
            .index
            .search(&query, neighbors)
            .map_err(|d| IndexSearchError::IndexSearchError(d))?;
        let indices: Vec<i64> = rs.labels.iter().map(|i| i.to_native()).collect();
        log::debug!("Index {:?}", start.elapsed());
        Ok(indices)
    }
}

#[derive(Debug)]
pub enum IndexLoadError {
    FileNotFound,
    IndexReadError(faiss::error::Error),
    IndexFormatError(faiss::error::Error),
}

#[derive(Debug)]
pub enum IndexSearchError {
    IncorrectDimensions,
    IndexSearchError(faiss::error::Error),
}
impl std::error::Error for IndexLoadError {}
impl std::error::Error for IndexSearchError {}

impl Display for IndexLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexLoadError::FileNotFound => write!(f, "SearchService: Index not found"),
            IndexLoadError::IndexReadError(err) => {
                write!(f, "SearchService: {}", err)
            }
            IndexLoadError::IndexFormatError(err) => {
                write!(f, "SearchService: {}", err)
            }
        }
    }
}

impl Display for IndexSearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexSearchError::IncorrectDimensions => {
                write!(f, "SearchService: Incorrect dimensions for search")
            }
            IndexSearchError::IndexSearchError(err) => {
                write!(f, "SearchService: {}", err)
            }
        }
    }
}
