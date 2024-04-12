use super::{
    cache::DocumentCache, document::Document, DocstoreRetrieveError, DocumentStore,
    DocumentStoreKind,
};

pub(super) trait DocumentDatabase: Send + Sync {
    async fn retreive_from_db(
        &self,
        indices: &[i64],
    ) -> Result<Vec<Document>, DocstoreRetrieveError>;
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
