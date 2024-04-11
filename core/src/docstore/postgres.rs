use crate::formatter::Provenance;
use chrono::DateTime;
use flate2::read::GzDecoder;
use sqlx::{postgres::PgPool, Postgres};
use std::io::Read;
use url::Url;

use super::{Docstore, DocstoreLoadError, DocstoreRetrieveError, DocumentStore};

impl DocumentStore for Docstore<Postgres> {
    async fn retreive(
        &self,
        indices: &[i64],
    ) -> Result<Vec<(usize, String, Provenance)>, DocstoreRetrieveError> {
        let start = std::time::Instant::now();

        let docs_rows = sqlx::query!(
            r#"
            SELECT document.id,
                document.text,
                article.title,
                article.access_date,
                article.modification_date
            FROM document
            INNER JOIN article ON document.article = article.id
            WHERE document.id IN
                (SELECT *
                FROM UNNEST($1::bigint[]))
            "#,
            &indices[..]
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| DocstoreRetrieveError::IndexOutOfRange)?;

        let docs = docs_rows
            .into_iter()
            .filter_map(|row| {
                let index = row.id;

                let binary_data = row.text.unwrap();
                let mut gz = GzDecoder::new(&*binary_data);
                let mut document = String::new();
                gz.read_to_string(&mut document).ok()?;

                let article_title = row.title.unwrap();
                let access_date = row.access_date.unwrap();
                let modification_date = row.modification_date.unwrap();

                let access_date = DateTime::from_timestamp_millis(access_date)?
                    .naive_utc()
                    .date();
                let modification_date = DateTime::from_timestamp_millis(modification_date)?
                    .naive_utc()
                    .date();

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
                    docs.iter().find(|d| d.0 == *docstore_index)?;
                Some((
                    array_index + 1,
                    doc_text.clone(),
                    document_provenance.clone(),
                ))
            })
            .collect::<Vec<(usize, String, Provenance)>>();

        log::debug!("SQL Query {:?}", start.elapsed());

        Ok(result)
    }
}

impl Docstore<Postgres> {
    pub async fn new(docstore_path: &Url, redis_url: &Url) -> Result<Self, DocstoreLoadError> {
        let docstore_path = docstore_path.as_ref();
        let pool = PgPool::connect(docstore_path)
            .await
            .map_err(DocstoreLoadError::Database)?;

        let cache = redis::Client::open(redis_url.to_string()).map_err(DocstoreLoadError::Redis)?;

        Ok(Docstore { pool, cache })
    }
}
