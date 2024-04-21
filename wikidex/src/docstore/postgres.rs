use std::io::Read;

use crate::{docstore::document::Document, formatter::Provenance};
use chrono::DateTime;
use flate2::read::GzDecoder;
use sqlx::{postgres::PgPool, Postgres};
use url::Url;

use super::{database::DocumentDatabase, Docstore, DocstoreLoadError, DocstoreRetrieveError};

impl DocumentDatabase for Docstore<Postgres> {
    async fn retreive_from_db(
        &self,
        indices: &[i64],
    ) -> Result<Vec<Document>, DocstoreRetrieveError> {
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
            .filter_map(|docstore_index| {
                let (index, doc_text, document_provenance) =
                    docs.iter().find(|d| d.0 == *docstore_index)?;
                Some(Document {
                    index: *index,
                    text: doc_text.clone(),
                    provenance: document_provenance.clone(),
                })
            })
            .collect::<Vec<Document>>();

        Ok(result)
    }
}

impl Docstore<Postgres> {
    pub(crate) async fn new(
        docstore_path: &Url,
        redis_url: &Url,
    ) -> Result<Self, DocstoreLoadError> {
        let docstore_path = docstore_path.as_ref();
        let pool = PgPool::connect(docstore_path).await?;
        let client = redis::Client::open(redis_url.to_string())?;
        let cache = client.get_multiplexed_tokio_connection().await?;
        Ok(Docstore { pool, cache })
    }
}
