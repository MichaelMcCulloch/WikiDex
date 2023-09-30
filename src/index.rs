use std::{
    fmt::{Debug, Display, Formatter},
    path::{Path, PathBuf},
};

use faiss::{FlatIndex, index::flat::FlatIndexImpl};

pub struct Index {
    index: FlatIndexImpl,
    dims: u32,
}

pub enum IndexLoadError {
    FileNotFound,
    IndexReadError(faiss::error::Error),
    IndexFormatError(faiss::error::Error)
}

pub struct IndexSearchError;

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
        let index_path = PathBuf::from(index_path.as_ref());
        if !index_path.exists() {
            return Err(IndexLoadError::FileNotFound);
        }

        let index = faiss::read_index(index_path.to_str().unwrap())
            .map_err(|e| IndexLoadError::IndexReadError(e))?
            .into_flat()
            .map_err(|e| IndexLoadError::IndexFormatError(e))?;
        let dims = faiss::Index::d(&index);
        Ok(Index {index, dims })
    }
}


pub trait Search {
    type E;
    fn search(query: &Vec<Vec<f32>>, count: u32)-> Result<(), IndexSearchError>{
        Ok(())
    }
}