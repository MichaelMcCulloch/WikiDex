use redis::AsyncCommands;
use sqlx::Database;

use super::{document::Document, Docstore, DocstoreRetrieveError, DocumentStoreImpl};

pub(super) trait DocumentCache: Send + Sync {
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

impl DocumentCache for DocumentStoreImpl {
    async fn insert_into_cache(
        &self,
        index: i64,
        data: Document,
    ) -> Result<(), DocstoreRetrieveError> {
        match self {
            #[cfg(feature = "postgres")]
            DocumentStoreImpl::Postgres(docstore) => docstore.insert_into_cache(index, data).await,
            #[cfg(feature = "sqlite")]
            DocumentStoreImpl::Sqlite(docstore) => docstore.insert_into_cache(index, data).await,
        }
    }

    async fn retreive_from_cache(
        &self,
        indices: &[i64],
    ) -> Result<(Vec<Document>, Vec<i64>), DocstoreRetrieveError> {
        match self {
            #[cfg(feature = "postgres")]
            DocumentStoreImpl::Postgres(docstore) => docstore.retreive_from_cache(indices).await,
            #[cfg(feature = "sqlite")]
            DocumentStoreImpl::Sqlite(docstore) => docstore.retreive_from_cache(indices).await,
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
            .await?;
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
        tokio::spawn(async move {
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
