mod document;
mod error;

#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

use actix_web::rt;
pub(crate) use error::{DocstoreLoadError, DocstoreRetrieveError};

use redis::{aio::MultiplexedConnection, AsyncCommands};
use sqlx::{Database, Pool};

#[cfg(feature = "postgres")]
use sqlx::Postgres;
#[cfg(feature = "sqlite")]
use sqlx::Sqlite;

use self::document::Document;

pub(crate) struct Docstore<DB: Database> {
    cache: MultiplexedConnection,
    pool: Pool<DB>,
}

pub(crate) enum DocumentStoreKind {
    #[cfg(feature = "postgres")]
    Postgres(Docstore<Postgres>),
    #[cfg(feature = "sqlite")]
    Sqlite(Docstore<Sqlite>),
}

pub(crate) trait DocumentStore: Send + Sync + DocumentDatabase {
    async fn retreive(&self, indices: &[i64]) -> Result<Vec<Document>, DocstoreRetrieveError>;
}
pub(crate) trait DocumentDatabase: Send + Sync {
    async fn retreive_from_db(
        &self,
        indices: &[i64],
    ) -> Result<Vec<Document>, DocstoreRetrieveError>;
}
pub(crate) trait DocumentCache: Send + Sync {
    async fn insert_into_cache(
        &self,
        index: i64,
        data: Document,
    ) -> Result<(), DocstoreRetrieveError>;
    async fn retreive_from_cache(
        &self,
        indices: &[i64],
    ) -> Result<(Vec<Document>, Vec<i64>), DocstoreRetrieveError>;
}

impl DocumentStore for DocumentStoreKind {
    async fn retreive(&self, indices: &[i64]) -> Result<Vec<Document>, DocstoreRetrieveError> {
        match self {
            #[cfg(feature = "postgres")]
            DocumentStoreKind::Postgres(docstore) => docstore.retreive(indices).await,
            #[cfg(feature = "sqlite")]
            DocumentStoreKind::Sqlite(docstore) => docstore.retreive(indices).await,
        }
    }
}

impl DocumentDatabase for DocumentStoreKind {
    async fn retreive_from_db(
        &self,
        indices: &[i64],
    ) -> Result<Vec<Document>, DocstoreRetrieveError> {
        match self {
            #[cfg(feature = "postgres")]
            DocumentStoreKind::Postgres(docstore) => docstore.retreive_from_db(indices).await,
            #[cfg(feature = "sqlite")]
            DocumentStoreKind::Sqlite(docstore) => docstore.retreive_from_db(indices).await,
        }
    }
}

impl<DB: Database> DocumentCache for Docstore<DB> {
    async fn retreive_from_cache(
        &self,
        indices: &[i64],
    ) -> Result<(Vec<Document>, Vec<i64>), DocstoreRetrieveError> {
        let mut cache = self.cache.clone();
        let result: Vec<Option<Document>> = redis::cmd("MGET")
            .arg(indices)
            .query_async(&mut cache)
            .await
            .map_err(DocstoreRetrieveError::Redis)?;
        let hits = result.into_iter().flatten().collect::<Vec<_>>();
        if hits.is_empty() {
            log::debug!("Cache Miss: {indices:?}");
            return Ok((vec![], indices.to_vec()));
        }

        let cache_hits = hits.iter().map(|d| d.index).collect::<Vec<_>>();
        let cache_misses = indices
            .iter()
            .filter_map(|index| {
                if cache_hits.contains(index) {
                    None
                } else {
                    Some(*index)
                }
            })
            .collect::<Vec<_>>();
        if !cache_misses.is_empty() {
            log::debug!("Cache Miss: {cache_misses:?}");
        }
        Ok((hits, cache_misses))
    }
    async fn insert_into_cache(
        &self,
        index: i64,
        data: Document,
    ) -> Result<(), DocstoreRetrieveError> {
        let mut cache = self.cache.clone();
        rt::spawn(async move {
            let result: Result<(), DocstoreRetrieveError> = cache
                .set(index, data)
                .await
                .map_err(DocstoreRetrieveError::Redis);
            if let Err(e) = result {
                log::error!("{e}");
                Err(e)
            } else {
                Ok::<(), DocstoreRetrieveError>(())
            }
        });

        Ok(())
    }
}
impl<T> DocumentStore for T
where
    T: DocumentDatabase + DocumentCache,
{
    async fn retreive(&self, indices: &[i64]) -> Result<Vec<Document>, DocstoreRetrieveError> {
        let (cached_documents, cache_misses) = self.retreive_from_cache(indices).await?;

        let missed_documents = if !cache_misses.is_empty() {
            let documents = self.retreive_from_db(&cache_misses).await?;
            for document in documents.iter() {
                let result = self
                    .insert_into_cache(document.index, document.clone())
                    .await;
                if let Err(e) = result {
                    log::error!("{e}")
                }
            }
            documents
        } else {
            vec![]
        };

        let mut documents = vec![];
        documents.extend(cached_documents);
        documents.extend(missed_documents);

        Ok(documents)
    }
}
