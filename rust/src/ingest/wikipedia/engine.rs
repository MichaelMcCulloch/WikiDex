use super::{
    helper::{self as h, text::RecursiveCharacterTextSplitter},
    IngestError::{self, *},
    WikiMarkupProcessor,
};
use crate::openai::OpenAiDelegate;
use indicatif::MultiProgress;
use sqlx::SqlitePool;
use tokio::sync::mpsc::unbounded_channel;

use std::{fs::File, io::BufReader, path::Path, sync::Arc};

const MARKUP_DB_NAME: &str = "wikipedia_markup.sqlite";
const DOCSTORE_DB_NAME: &str = "wikipedia_docstore.sqlite";
const VECTOR_TMP_DB_NAME: &str = "wikipedia_index.sqlite";
const VECTOR_INDEX_NAME: &str = "wikipedia_index.faiss";

const PCA_DIMENSIONS: usize = 128;
const MINIMUM_PASSAGE_LENGTH_IN_WORDS: usize = 15;

pub(crate) struct Engine {
    openai: Arc<OpenAiDelegate>,
    markup_processor: WikiMarkupProcessor,
    text_splitter: RecursiveCharacterTextSplitter<'static>,
    multi_progress: MultiProgress,
}

impl Engine {
    pub(crate) fn new(
        openai: OpenAiDelegate,
        multi_progress: MultiProgress,
        chunk_size: usize,
        chunk_overlap: usize,
    ) -> Self {
        let markup_processor = WikiMarkupProcessor::new();

        Self {
            openai: Arc::new(openai),
            markup_processor,
            multi_progress,
            text_splitter: RecursiveCharacterTextSplitter::new(
                chunk_size,
                chunk_overlap,
                None,
                true,
            ),
        }
    }

    async fn create_markup_database<P: AsRef<Path>>(
        &self,
        input_xml: &P,
        pool: &SqlitePool,
    ) -> Result<i64, IngestError> {
        let access_date = h::wiki::get_date_from_xml_name(input_xml)?;
        let file = BufReader::with_capacity(
            2 * 1024 * 1024,
            File::open(input_xml.as_ref()).map_err(IoError)?,
        );

        let eligible_pages_bar = h::progress::new_progress_bar(&self.multi_progress, 7000000);
        let eligible_pages = h::wiki::get_eligible_pages(file, &eligible_pages_bar);

        let pages_compressed_bar =
            h::progress::new_progress_bar(&self.multi_progress, eligible_pages.len() as u64);
        let pages_compressed = h::wiki::compress_articles(eligible_pages, &pages_compressed_bar);

        let article_count = pages_compressed.len();
        let markup_written_bar =
            h::progress::new_progress_bar(&self.multi_progress, article_count as u64);
        let articles_written =
            h::sql::populate_markup_db(pool, pages_compressed, access_date, &markup_written_bar)
                .await?;

        h::sql::write_completion_timestamp(pool, articles_written).await?;
        Ok(articles_written)
    }

    async fn create_docstore_database(
        &self,
        markup_pool: SqlitePool,
        docstore_pool: &SqlitePool,
    ) -> Result<i64, IngestError> {
        let document_count = h::sql::count_elements(&markup_pool).await?.ok_or(NoRows)?;
        //Obtain markup
        let obtain_markup_bar =
            h::progress::new_progress_bar(&self.multi_progress, document_count as u64);
        let pages = h::sql::obtain_markup(&markup_pool, &obtain_markup_bar).await?;

        let pages_decompressed_bar =
            h::progress::new_progress_bar(&self.multi_progress, pages.len() as u64);
        let documents = h::wiki::decompress_articles_into_documents(
            pages,
            &pages_decompressed_bar,
            &self.markup_processor,
            &self.text_splitter,
            MINIMUM_PASSAGE_LENGTH_IN_WORDS,
        );

        let docstore_written_bar =
            h::progress::new_progress_bar(&self.multi_progress, documents.len() as u64);
        let documents_written =
            h::sql::populate_docstore_db(docstore_pool, documents, &docstore_written_bar).await?;
        //process it in parrallel
        h::sql::write_completion_timestamp(docstore_pool, documents_written).await?;
        //write it to the database
        Ok(documents_written)
    }
    async fn create_temp_vector_database(
        &self,
        docstore_pool: SqlitePool,
        tmp_vector_pool: &SqlitePool,
    ) -> Result<i64, IngestError> {
        let document_count = h::sql::count_elements(&docstore_pool)
            .await?
            .ok_or(NoRows)?;
        let create_vectors_bar =
            h::progress::new_progress_bar(&self.multi_progress, document_count as u64);
        let (tx, rx) = unbounded_channel::<(i64, Vec<f32>)>();

        let tmp_vector_pool_clone = Arc::new(tmp_vector_pool.clone());
        let create_vectors_bar_clone = Arc::new(create_vectors_bar.clone());

        create_vectors_bar.set_message("Writing vectorstore to DB...");

        let db_writter_thread = actix_web::rt::spawn(async move {
            h::sql::write_vectorstore(rx, tmp_vector_pool_clone, create_vectors_bar_clone).await
        });

        h::sql::populate_vectorstore_db(self.openai.clone(), &docstore_pool, document_count, tx)
            .await?;
        h::sql::write_completion_timestamp(tmp_vector_pool, document_count).await?;
        create_vectors_bar.set_message("Writing vectorstore to DB...DONE");

        let _ = db_writter_thread.await;

        Ok(document_count)
    }

    async fn create_vector_index<P: AsRef<Path>>(
        &self,
        tmp_vector_pool: &SqlitePool,
        index_path: &P,
    ) -> Result<usize, IngestError> {
        let vector_count = h::sql::count_elements(tmp_vector_pool)
            .await?
            .ok_or(NoRows)?;
        //Obtain markup
        let obtain_vectors_bar =
            h::progress::new_progress_bar(&self.multi_progress, vector_count as u64);
        let index_path = index_path.as_ref();
        let vector_embeddings =
            h::sql::obtain_vectors(tmp_vector_pool, &obtain_vectors_bar).await?;
        drop(obtain_vectors_bar);
        let count = vector_embeddings.len();
        h::faiss::populate_vectorestore_index(&index_path, vector_embeddings, PCA_DIMENSIONS)?;
        Ok(count)
    }
}

impl Engine {
    pub(crate) async fn ingest_wikipedia(
        self,
        input_xml: &Path,
        output_directory: &Path,
    ) -> Result<usize, IngestError> {
        match (input_xml.exists(), output_directory.exists()) {
            (true, false) => Err(DirectoryNotFound(output_directory.to_path_buf())),
            (false, _) => Err(XmlNotFound(input_xml.to_path_buf())),
            (true, true) => {
                let markup_db_path = output_directory.join(MARKUP_DB_NAME);
                let markup_pool = h::sql::get_sqlite_pool(&markup_db_path).await?;

                if !h::sql::database_is_complete(&markup_pool).await? {
                    log::info!("Preparing markup DB...");
                    self.create_markup_database(&input_xml, &markup_pool)
                        .await?;
                }
                log::info!("Markup DB is ready at {}", markup_db_path.display());

                let docstore_db_path = output_directory.join(DOCSTORE_DB_NAME);
                let docstore_pool = h::sql::get_sqlite_pool(&docstore_db_path).await?;

                if !h::sql::database_is_complete(&docstore_pool).await? {
                    log::info!("Preparing docstore DB...");

                    self.create_docstore_database(markup_pool, &docstore_pool)
                        .await?;
                }
                log::info!("Docstore DB is ready at {}", docstore_db_path.display());

                let tmp_vector_db_path = output_directory.join(VECTOR_TMP_DB_NAME);
                let tmp_vector_pool = h::sql::get_sqlite_pool(&tmp_vector_db_path).await?;
                if !h::sql::database_is_complete(&tmp_vector_pool).await? {
                    log::info!("Preparing Vector DB...");

                    self.create_temp_vector_database(docstore_pool, &tmp_vector_pool)
                        .await?;
                }
                log::info!("Vector DB is ready at {}", tmp_vector_db_path.display());

                let index_path = output_directory.join(VECTOR_INDEX_NAME);
                if !h::faiss::index_is_complete(&index_path).map_err(IndexError)? {
                    log::info!("Preparing Vector Index...");

                    self.create_vector_index(&tmp_vector_pool, &index_path)
                        .await?;
                }
                log::info!("Vector Index is ready at {}", index_path.display());

                Ok(1)
            }
        }
    }
}
