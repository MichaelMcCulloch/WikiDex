use std::{path::Path, io::Read};
use std::
    fmt::{self, Display, Formatter, Debug};

use flate2::read::GzDecoder;
use sqlx::{sqlite::SqlitePool, Row};
pub struct SqliteDocstore {
    pool: SqlitePool,
}

pub enum DocstoreLoadError {
    FileNotFound,
}
pub enum DocstoreRetrieveError {
    IndexOutOfRange,
    InvalidDocument,
}


impl Display for DocstoreLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DocstoreLoadError::FileNotFound => write!(f, "File not found"),
        }
    }
}

impl Debug for DocstoreLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for DocstoreRetrieveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DocstoreRetrieveError::IndexOutOfRange => write!(f, "Index out of range"),
            DocstoreRetrieveError::InvalidDocument => write!(f, "Invalid document"),
        }
    }
}

impl Debug for DocstoreRetrieveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl SqliteDocstore {
    pub async fn new<P: AsRef<Path>>(docstore_path: &P) -> Result<Self, DocstoreLoadError> {
        let start = std::time::Instant::now();
        let docstore_path = docstore_path.as_ref();
        if !docstore_path.exists() {
            return Err(DocstoreLoadError::FileNotFound);
        }
        let pool =
        SqlitePool::connect(&docstore_path.to_str().unwrap()).await
                .map_err(|_| DocstoreLoadError::FileNotFound)?;
        log::info!("Load Docstore {:?}", start.elapsed());
        Ok(SqliteDocstore { pool })
    }
}
#[async_trait::async_trait]
pub trait Docstore {
    type E;
    async fn retreive(&self, indices: &Vec<Vec<i64>>) -> Result<Vec<Vec<String>>, Self::E>;
}

#[async_trait::async_trait]
impl Docstore for SqliteDocstore {
    type E = DocstoreRetrieveError;

    async fn retreive(&self, indices: &Vec<Vec<i64>>) -> Result<Vec<Vec<String>>, DocstoreRetrieveError> {

        let start = std::time::Instant::now();
        let flattened_indices: Vec<i64> = indices.into_iter().flatten().map(|i| *i).collect();

        // build dynamic query statement
        let ids = flattened_indices 
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let query = format!("SELECT id, doc FROM documents WHERE id IN ({})", ids);
        let docs_rows = sqlx::query(&query).fetch_all(&self.pool).await
            .map_err(|_| DocstoreRetrieveError::IndexOutOfRange)?;

        let docs: Vec<(i64, String)> = docs_rows
            .into_iter()
            .map(|row| {

                let index = row.get::<i64, _>("id");
                let binary_data = row.get::<Vec<u8>, _>("doc");
                let mut gz = GzDecoder::new(&*binary_data);
                let mut document = String::new();
                gz.read_to_string(&mut document).unwrap();
                (
                    index,
                    document,
                )
            })
            .collect();

        let result = indices.iter().map(|is| {
            is.iter().map(|i| {
                let doc = docs.iter().filter(|d|d.0 == *i).next().unwrap();
                doc.1.clone()
            }).collect::<Vec<String>>()
        }).collect::<Vec<Vec<String>>>();
      
        log::info!("SQL Query {:?}", start.elapsed());

      
        Ok(result)
    }
}
