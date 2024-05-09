use faiss::{index_factory, Index, MetricType};

use sqlx::{
    migrate::MigrateDatabase,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Sqlite, SqlitePool,
};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

const PCA_DIMENSIONS: usize = 128;
const EMBEDDING_DIMENSIONS: u32 = 384;

async fn obtain_vectors(pool: &SqlitePool) -> anyhow::Result<Vec<Vec<f32>>> {
    let mut connection = pool.acquire().await?;

    let records = sqlx::query!("SELECT gte_small FROM embeddings ORDER BY id ASC;")
        .map(|record| {
            let embedding_bytes = record.gte_small;
            let mut embedding: Vec<f32> = vec![];
            for f32_bytes in embedding_bytes.chunks_exact(4) {
                let mut b = [0u8; 4];
                b.copy_from_slice(f32_bytes);
                embedding.push(f32::from_le_bytes(b));
            }
            embedding
        })
        .fetch_all(&mut *connection)
        .await?;

    Ok(records)
}
async fn create_vector_index(
    tmp_vector_pool: &SqlitePool,
    index_path: &PathBuf,
) -> anyhow::Result<usize> {
    let vector_embeddings = obtain_vectors(tmp_vector_pool).await?;
    let count = vector_embeddings.len();
    populate_vectorestore_index(&index_path, vector_embeddings, PCA_DIMENSIONS)?;
    Ok(count)
}

async fn create_index(sqlite_path: PathBuf, index_path: PathBuf) -> anyhow::Result<()> {
    if !Sqlite::database_exists(sqlite_path.to_str().unwrap()).await? {
        Sqlite::create_database(sqlite_path.to_str().unwrap()).await?;
    }

    let options = SqliteConnectOptions::new();

    let docstore_option = options.filename(sqlite_path);

    let docstore_pool = SqlitePoolOptions::new()
        .acquire_timeout(Duration::from_secs(10000))
        .max_connections(1)
        .connect_with(docstore_option)
        .await?;
    create_vector_index(&docstore_pool, &index_path)
        .await
        .unwrap();
    Ok(())
}

fn populate_vectorestore_index<P: AsRef<Path>>(
    index_path: &P,
    vector_embeddings: Vec<Vec<f32>>,
    pca_dimensions: usize,
) -> anyhow::Result<()> {
    let vector_contiguous = vector_embeddings.into_iter().flatten().collect::<Vec<_>>();

    let mut index = index_factory(
        EMBEDDING_DIMENSIONS,
        format!("PCA{pca_dimensions},Flat"),
        MetricType::L2,
    )?;

    log::info!("Training Vectorstore. Takes up to 10 minutes...");
    index.train(&vector_contiguous)?;

    log::info!("Adding vectors to vectorstore. Takes up to an hour...");
    index.add(&vector_contiguous)?;

    log::info!("Writing vectorstore to disk. Please wait...");
    faiss::write_index(&index, index_path.as_ref().to_path_buf().to_str().unwrap())?;
    Ok(())
}
#[cfg(test)]
mod test {
    use super::create_index;
    use std::path::PathBuf;
    #[tokio::test]
    async fn test() {
        create_index(
            PathBuf::from("/home/michael/Documents/WIKIDUMPS/YYYYMMDD/wikipedia_index.sqlite"),
            PathBuf::from("/home/michael/Documents/WIKIDUMPS/YYYYMMDD/index/thenlper/gte-small/wikipedia_index.faiss"),
        )
        .await
        .unwrap();
    }
}
