// TODO: Move me to ingest::, rather than ingest::wikipedia::helper::.

use std::path::Path;

use faiss::{index_factory, Index, MetricType};

use crate::{
    index::IndexError,
    ingest::wikipedia::IngestError::{self, FaissError},
};

pub(crate) fn populate_vectorestore_index<P: AsRef<Path>>(
    index_path: &P,
    vector_embeddings: Vec<Vec<f32>>,
    pca_dimensions: usize,
) -> Result<(), IngestError> {
    let vector_contiguous = vector_embeddings
        .into_iter()
        .flat_map(|f| f)
        .collect::<Vec<_>>();

    let mut index = index_factory(384, format!("PCA{pca_dimensions},Flat"), MetricType::L2)
        .map_err(FaissError)?;

    log::info!("Training Vectorstore. Takes up to 10 minutes...");
    index.train(&vector_contiguous).map_err(FaissError)?;

    log::info!("Adding vectors to vectorstore. Takes up to an hour...");
    index.add(&vector_contiguous).map_err(FaissError)?;

    log::info!("Writing vectorstore to disk. Please wait...");
    faiss::write_index(
        &index,
        index_path
            .as_ref()
            .to_path_buf()
            .to_str()
            .ok_or(IngestError::DirectoryNotFound(
                index_path.as_ref().to_path_buf(),
            ))?,
    )
    .map_err(FaissError)?;
    Ok(())
}

pub(crate) fn index_is_complete<P: AsRef<Path>>(index_path: &P) -> Result<bool, IndexError> {
    if index_path.as_ref().exists() {
        let index = faiss::read_index(
            index_path
                .as_ref()
                .to_str()
                .ok_or(IndexError::FileNotFound)?,
        )
        .map_err(IndexError::IndexReadError)?
        .into_pre_transform()
        .map_err(IndexError::IndexFormatError)?;
        Ok(index.is_trained())
    } else {
        Ok(false)
    }
}
