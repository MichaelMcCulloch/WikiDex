use crate::formatter::Provenance;
use chrono::NaiveDateTime;
use flate2::read::GzDecoder;
use sqlx::{sqlite::SqlitePool, Row};
use std::{io::Read, path::Path};

use super::{DocstoreLoadError, DocstoreRetrieveError, DocumentService};
pub struct SqliteDocstore {
    pool: SqlitePool,
}

impl SqliteDocstore {
    pub async fn new<P: AsRef<Path>>(docstore_path: &P) -> Result<Self, DocstoreLoadError> {
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
        Ok(SqliteDocstore { pool })
    }
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
            "SELECT document.id, document.text, article.title, article.access_date, article.modification_date FROM document INNER JOIN article ON document.article = article.id WHERE document.id IN ({})",
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

                let binary_data = row.get::<Vec<u8>, _>("text");
                let mut gz = GzDecoder::new(&*binary_data);
                let mut document = String::new();
                gz.read_to_string(&mut document).ok()?;

                let article_title = row.get::<String, _>("title");

                let access_date = row.get::<i64, _>("access_date");
                let modification_date = row.get::<i64, _>("modification_date");

                let access_date = NaiveDateTime::from_timestamp_millis(access_date)?.date();
                let modification_date =
                    NaiveDateTime::from_timestamp_millis(modification_date)?.date();

                let provenance =
                    Provenance::Wikipedia(article_title, access_date, modification_date);
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
                        Some((
                            array_index + 1,
                            doc_text.clone(),
                            document_provenance.clone(),
                        ))
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

        let query = format!("SELECT document.id, document.text, article.title, article.access_date, article.modification_date FROM document INNER JOIN article ON document.article = article.id WHERE document.id IN ({})", ids);

        let docs_rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|_| DocstoreRetrieveError::IndexOutOfRange)?;

        let docs = docs_rows
            .into_iter()
            .filter_map(|row| {
                let index = row.get::<i64, _>("id");

                let binary_data = row.get::<Vec<u8>, _>("text");
                let mut gz = GzDecoder::new(&*binary_data);
                let mut document = String::new();
                gz.read_to_string(&mut document).ok()?;

                let article_title = row.get::<String, _>("title");
                let access_date = row.get::<i64, _>("access_date");
                let modification_date = row.get::<i64, _>("modification_date");

                let access_date = NaiveDateTime::from_timestamp_millis(access_date)?.date();
                let modification_date =
                    NaiveDateTime::from_timestamp_millis(modification_date)?.date();

                let provenance =
                    Provenance::Wikipedia(article_title, access_date, modification_date);
                Some((index, document, provenance))
            })
            .collect::<Vec<(i64, String, Provenance)>>();

        let result = indices
            .iter()
            .enumerate()
            .filter_map(|(array_index, docstore_index)| {
                let (_, doc_text, document_provenance) =
                    docs.iter().filter(|d| d.0 == *docstore_index).next()?;
                Some((
                    array_index + 1,
                    doc_text.clone(),
                    document_provenance.clone(),
                ))
                // No excuse but being lazy. Docs will always be a set, and it's one to one with the query.
            })
            .collect::<Vec<(usize, String, Provenance)>>();

        log::debug!("SQL Query {:?}", start.elapsed());

        Ok(result)
    }
}
