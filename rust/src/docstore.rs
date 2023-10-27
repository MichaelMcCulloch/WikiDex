use std::fmt::{self, Debug, Display, Formatter};
use std::{io::Read, path::Path};

use flate2::read::GzDecoder;
use sqlx::{sqlite::SqlitePool, Row};
pub struct SqliteDocstore {
    pool: SqlitePool,
}

impl SqliteDocstore {
    pub async fn new<P: AsRef<Path>>(docstore_path: &P) -> Result<Self, DocstoreLoadError> {
        let start = std::time::Instant::now();
        let docstore_path = docstore_path.as_ref();
        if !docstore_path.exists() {
            return Err(DocstoreLoadError::FileNotFound);
        }
        let pool = SqlitePool::connect(
            &docstore_path
                .to_str()
                .expect("Docstore path is not a string"),
        )
        .await
        .map_err(|_| DocstoreLoadError::FileNotFound)?;
        log::info!("Load Docstore {:?}", start.elapsed());
        Ok(SqliteDocstore { pool })
    }
}
#[async_trait::async_trait]
pub trait DocumentService {
    type E;
    async fn retreive_batch(
        &self,
        indices: &Vec<Vec<i64>>,
    ) -> Result<Vec<Vec<(usize, String)>>, Self::E>;
    async fn retreive(&self, indices: &Vec<i64>) -> Result<Vec<(usize, String)>, Self::E>;
}

#[async_trait::async_trait]
impl DocumentService for SqliteDocstore {
    type E = DocstoreRetrieveError;

    async fn retreive_batch(
        &self,
        indices: &Vec<Vec<i64>>,
    ) -> Result<Vec<Vec<(usize, String)>>, Self::E> {
        let start = std::time::Instant::now();
        let flattened_indices: Vec<i64> = indices.into_iter().flatten().map(|i| *i).collect();

        // build dynamic query statement
        let ids = flattened_indices
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let query = format!("SELECT id, doc FROM document WHERE id IN ({})", ids);
        let docs_rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DocstoreRetrieveError::SqlxError(e))?;

        let docs: Vec<(i64, String)> = docs_rows
            .into_iter()
            .filter_map(|row| {
                let index = row.get::<i64, _>("id");
                let binary_data = row.get::<Vec<u8>, _>("doc");
                let mut gz = GzDecoder::new(&*binary_data);
                let mut document = String::new();
                gz.read_to_string(&mut document).ok()?;
                Some((index, document))
            })
            .collect();

        let result = indices
            .iter()
            .map(|is| {
                is.iter()
                    .enumerate()
                    .filter_map(|(array_index, docstore_index)| {
                        let doc = docs.iter().filter(|d| d.0 == *docstore_index).next()?;
                        Some((array_index, doc.1.clone()))
                    })
                    .collect::<Vec<(usize, String)>>()
            })
            .collect::<Vec<Vec<(usize, String)>>>();

        log::debug!("SQL Query {:?}", start.elapsed());

        Ok(result)
    }

    async fn retreive(&self, indices: &Vec<i64>) -> Result<Vec<(usize, String)>, Self::E> {
        let start = std::time::Instant::now();

        // build dynamic query statement
        let ids = indices
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let query = format!("SELECT id, doc FROM document WHERE id IN ({})", ids);
        let docs_rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|_| DocstoreRetrieveError::IndexOutOfRange)?;

        let docs: Vec<(i64, String)> = docs_rows
            .into_iter()
            .filter_map(|row| {
                let index = row.get::<i64, _>("id");
                let binary_data = row.get::<Vec<u8>, _>("doc");
                let mut gz = GzDecoder::new(&*binary_data);
                let mut document = String::new();
                gz.read_to_string(&mut document).ok()?;
                Some((index, document))
            })
            .collect();

        let result = indices
            .iter()
            .enumerate()
            .filter_map(|(array_index, docstore_index)| {
                let doc = docs.iter().filter(|d| d.0 == *docstore_index).next()?;
                Some((array_index, doc.1.clone()))
            })
            .collect::<Vec<(usize, String)>>();

        log::debug!("SQL Query {:?}", start.elapsed());

        Ok(result)
    }
}

#[derive(Debug)]
pub enum DocstoreLoadError {
    FileNotFound,
}
#[derive(Debug)]
pub enum DocstoreRetrieveError {
    IndexOutOfRange,
    InvalidDocument,
    SqlxError(sqlx::error::Error),
}

impl std::error::Error for DocstoreLoadError {}
impl std::error::Error for DocstoreRetrieveError {}

impl Display for DocstoreLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DocstoreLoadError::FileNotFound => write!(f, "DocumentService: File not found"),
        }
    }
}

impl Display for DocstoreRetrieveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DocstoreRetrieveError::IndexOutOfRange => {
                write!(f, "DocumentService: Index out of range")
            }
            DocstoreRetrieveError::InvalidDocument => {
                write!(f, "DocumentService: Invalid document")
            }
            DocstoreRetrieveError::SqlxError(e) => {
                write!(f, "DocumentService: {e}")
            }
        }
    }
}
