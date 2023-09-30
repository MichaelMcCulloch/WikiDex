use std::{
    fmt::{Debug, Display, Formatter},
    path::{Path, PathBuf},
};

use faiss::{ConcurrentIndex, FlatIndex};

pub struct Index {
    index: FlatIndex,
    dims: u32,
}

pub enum IndexLoadError {
    FileNotFound,
    IndexReadError(faiss::error::Error),
    IndexFormatError(faiss::error::Error),
}

pub enum IndexSearchError {
    IncorrectDimensions,
}

impl Display for IndexLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Index Load Error")
    }
}
impl Display for IndexSearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Index Load Error")
    }
}

impl Debug for IndexLoadError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Debug for IndexSearchError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
impl Index {
    pub(crate) fn new<P: AsRef<Path>>(index_path: &P) -> Result<Self, IndexLoadError> {
        let start = std::time::Instant::now();

        let index_path = PathBuf::from(index_path.as_ref());
        if !index_path.exists() {
            return Err(IndexLoadError::FileNotFound);
        }

        let index = faiss::read_index(index_path.to_str().unwrap())
            .map_err(|e| IndexLoadError::IndexReadError(e))?
            .into_flat()
            .map_err(|e| IndexLoadError::IndexFormatError(e))?;
        let dims = faiss::Index::d(&index);

        log::info!("Load Index {:?}", start.elapsed());
        Ok(Index { index, dims })
    }
}

pub trait Search {
    type E;
    fn search(&self, query: &Vec<Vec<f32>>, neighbors: usize) -> Result<Vec<Vec<i64>>, Self::E>;
}

impl Search for Index {
    type E = IndexSearchError;

    fn search(&self, query: &Vec<Vec<f32>>, neighbors: usize) -> Result<Vec<Vec<i64>>, IndexSearchError> {
        let start = std::time::Instant::now();
        let flattened_query : Vec<f32> = query
            .iter()
            .all(|q| q.len() == self.dims as usize)
            .then(||query.into_iter().flatten().map(|f|*f).collect() )
            .ok_or(IndexSearchError::IncorrectDimensions)?;

        let rs = self.index.search(&flattened_query, 4).unwrap();
        let x : Vec<i64>= rs.labels.iter().map(|i| i.to_native()).collect();
        let indices = x.chunks_exact(neighbors).map(|v|v.to_vec()).collect();
        log::info!("Index {:?}", start.elapsed());
        Ok(indices)
    }
}
