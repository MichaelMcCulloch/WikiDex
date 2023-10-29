use std::fmt::{self, Debug, Display, Formatter};
use std::{io::Read, path::Path};

use chrono::NaiveDate;
use flate2::read::GzDecoder;
use sqlx::{sqlite::SqlitePool, Row};

use crate::provenance::Provenance;

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
pub(crate) trait DocumentService {
    type E;
    type R;
    async fn retreive_batch(&self, indices: &Vec<Vec<i64>>) -> Result<Vec<Self::R>, Self::E>;
    async fn retreive(&self, indices: &Vec<i64>) -> Result<Self::R, Self::E>;
}

#[async_trait::async_trait]
impl DocumentService for SqliteDocstore {
    type E = DocstoreRetrieveError;
    type R = Vec<(usize, String, Provenance)>;
    async fn retreive_batch(&self, indices: &Vec<Vec<i64>>) -> Result<Vec<Self::R>, Self::E> {
        let start = std::time::Instant::now();
        let flattened_indices = indices
            .into_iter()
            .flatten()
            .map(|i| *i)
            .collect::<Vec<i64>>();

        // build dynamic query statement
        let ids = flattened_indices
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "SELECT document.id, document.doc, article.title FROM document INNER JOIN article ON document.article = article.id WHERE document.id IN ({})",
            ids
        );
        let docs_rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DocstoreRetrieveError::SqlxError(e))?;

        let docs = docs_rows
            .into_iter()
            .filter_map(|row| {
                let index = row.get::<i64, _>("id");

                let binary_data = row.get::<Vec<u8>, _>("doc");
                let mut gz = GzDecoder::new(&*binary_data);
                let mut document = String::new();
                gz.read_to_string(&mut document).ok()?;

                let article_title = row.get::<String, _>("title");
                let provenance = Provenance::Wikipedia(
                    article_title,
                    replace_me_asap_wikipedia_article_access_date(),
                    replace_me_asap_wikipedia_article_modification_date(),
                );
                Some((index, document, provenance))
            })
            .collect::<Vec<(i64, String, Provenance)>>();

        let result = indices
            .iter()
            .map(|is| {
                is.iter()
                    .enumerate()
                    .filter_map(|(array_index, docstore_index)| {
                        let (_, doc_text, document_provenance) =
                            docs.iter().filter(|d| d.0 == *docstore_index).next()?;
                        Some((array_index, doc_text.clone(), document_provenance.clone()))
                        // Multiple independent queries may have returned the same document, must be cloned.
                    })
                    .collect::<Vec<(usize, String, Provenance)>>()
            })
            .collect::<Vec<Vec<(usize, String, Provenance)>>>();

        log::debug!("SQL Query {:?}", start.elapsed());

        Ok(result)
    }

    async fn retreive(&self, indices: &Vec<i64>) -> Result<Self::R, Self::E> {
        let start = std::time::Instant::now();

        // build dynamic query statement
        let ids = indices
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let query = format!("SELECT document.id, document.doc, article.title FROM document INNER JOIN article ON document.article = article.id WHERE document.id IN ({})", ids);

        let docs_rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|_| DocstoreRetrieveError::IndexOutOfRange)?;

        let docs = docs_rows
            .into_iter()
            .filter_map(|row| {
                let index = row.get::<i64, _>("id");

                let binary_data = row.get::<Vec<u8>, _>("doc");
                let mut gz = GzDecoder::new(&*binary_data);
                let mut document = String::new();
                gz.read_to_string(&mut document).ok()?;

                let article_title = row.get::<String, _>("title");
                let provenance = Provenance::Wikipedia(
                    article_title,
                    replace_me_asap_wikipedia_article_access_date(),
                    replace_me_asap_wikipedia_article_modification_date(),
                );
                Some((index, document, provenance))
            })
            .collect::<Vec<(i64, String, Provenance)>>();

        let result = indices
            .iter()
            .enumerate()
            .filter_map(|(array_index, docstore_index)| {
                let (_, doc_text, document_provenance) =
                    docs.iter().filter(|d| d.0 == *docstore_index).next()?;
                Some((array_index, doc_text.clone(), document_provenance.clone()))
                // No excuse but being lazy. Docs will always be a set, and it's one to one with the query.
            })
            .collect::<Vec<(usize, String, Provenance)>>();

        log::debug!("SQL Query {:?}", start.elapsed());

        Ok(result)
    }
}

// TODO, store the date of the dump/source in the db.
fn replace_me_asap_wikipedia_article_access_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2023, 10, 01).unwrap()
}
// TODO, store the last date of modification in the db.
fn replace_me_asap_wikipedia_article_modification_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2023, 10, 01).unwrap()
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
