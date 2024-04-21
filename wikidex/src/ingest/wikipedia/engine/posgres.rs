use crate::{
    embedding_client::EmbeddingClient,
    ingest::wikipedia::{
        engine::{DOCSTORE_DB_NAME, MARKUP_DB_NAME, VECTOR_INDEX_NAME, VECTOR_TMP_DB_NAME},
        helper::{self as h, text::RecursiveCharacterTextSplitter},
        IngestError::{self, *},
        WikiMarkupProcessor,
    },
    llm_client::LlmClientKind,
};

use indicatif::MultiProgress;
use sqlx::{PgPool, Postgres};
use std::{fs::File, io::BufReader, marker::PhantomData, path::Path, sync::Arc};
use tokio::sync::mpsc::unbounded_channel;

use super::{Engine, MINIMUM_PASSAGE_LENGTH_IN_WORDS, PCA_DIMENSIONS};

impl Engine<Postgres> {
    pub(crate) fn new(
        llm: LlmClientKind,
        embed: EmbeddingClient,
        multi_progress: MultiProgress,
        chunk_size: usize,
        chunk_overlap: usize,
    ) -> Self {
        let markup_processor = WikiMarkupProcessor::new();

        Self {
            llm: Arc::new(llm),
            embed: Arc::new(embed),
            markup_processor,
            multi_progress,
            text_splitter: RecursiveCharacterTextSplitter::new(
                chunk_size,
                chunk_overlap,
                None,
                true,
            ),
            _phantom: PhantomData,
        }
    }

    async fn create_markup_database<P: AsRef<Path>>(
        &self,
        input_xml: &P,
        pool: &PgPool,
        ingest_limit: usize,
    ) -> Result<i64, IngestError> {
        let access_date = h::wiki::get_date_from_xml_name(input_xml)?;
        let file = BufReader::with_capacity(
            2 * 1024 * 1024,
            File::open(input_xml.as_ref()).map_err(IoError)?,
        );

        let eligible_pages_bar = h::progress::new_progress_bar(&self.multi_progress, 7000000);
        let eligible_pages = h::wiki::get_eligible_pages(file, &eligible_pages_bar, ingest_limit);

        let pages_compressed_bar =
            h::progress::new_progress_bar(&self.multi_progress, eligible_pages.len() as u64);
        let pages_compressed = h::wiki::compress_articles(eligible_pages, &pages_compressed_bar);

        let article_count = pages_compressed.len();
        let markup_written_bar =
            h::progress::new_progress_bar(&self.multi_progress, article_count as u64);
        let articles_written = h::postgres::populate_markup_db(
            pool,
            pages_compressed,
            access_date,
            &markup_written_bar,
        )
        .await?;

        h::postgres::write_completion_timestamp(pool, articles_written).await?;
        Ok(articles_written)
    }

    async fn create_docstore_database(
        &self,
        markup_pool: PgPool,
        docstore_pool: &PgPool,
    ) -> Result<i64, IngestError> {
        let document_count = h::postgres::count_elements(&markup_pool)
            .await?
            .ok_or(NoRows)?;
        //Obtain markup
        let obtain_markup_bar =
            h::progress::new_progress_bar(&self.multi_progress, document_count as u64);
        let pages = h::postgres::obtain_markup(&markup_pool, &obtain_markup_bar).await?;

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
            h::postgres::populate_docstore_db(docstore_pool, documents, &docstore_written_bar)
                .await?;
        //process it in parrallel
        h::postgres::write_completion_timestamp(docstore_pool, documents_written).await?;
        //write it to the database
        Ok(documents_written)
    }
    async fn create_temp_vector_database(
        &self,
        docstore_pool: PgPool,
        tmp_vector_pool: &PgPool,
    ) -> Result<i64, IngestError> {
        let document_count = h::postgres::count_elements(&docstore_pool)
            .await?
            .ok_or(NoRows)?;
        let create_vectors_bar =
            h::progress::new_progress_bar(&self.multi_progress, document_count as u64);
        let (tx, rx) = unbounded_channel::<Vec<(i64, Vec<f32>)>>();

        let tmp_vector_pool_clone = Arc::new(tmp_vector_pool.clone());
        let create_vectors_bar_clone = Arc::new(create_vectors_bar.clone());

        create_vectors_bar.set_message("Writing vectorstore to DB...");

        let db_writter_thread = actix_web::rt::spawn(async move {
            h::postgres::write_vectorstore(rx, tmp_vector_pool_clone, create_vectors_bar_clone)
                .await
        });

        h::postgres::populate_vectorstore_db(
            self.llm.clone(),
            self.embed.clone(),
            &docstore_pool,
            document_count,
            tx,
        )
        .await?;
        h::postgres::write_completion_timestamp(tmp_vector_pool, document_count).await?;
        create_vectors_bar.set_message("Writing vectorstore to DB...DONE");

        let _ = db_writter_thread.await;

        Ok(document_count)
    }

    async fn create_vector_index<P: AsRef<Path>>(
        &self,
        tmp_vector_pool: &PgPool,
        index_path: &P,
    ) -> Result<usize, IngestError> {
        let vector_count = h::postgres::count_elements(tmp_vector_pool)
            .await?
            .ok_or(NoRows)?;
        //Obtain markup
        let obtain_vectors_bar =
            h::progress::new_progress_bar(&self.multi_progress, vector_count as u64);
        let index_path = index_path.as_ref();
        let vector_embeddings =
            h::postgres::obtain_vectors(tmp_vector_pool, &obtain_vectors_bar).await?;
        drop(obtain_vectors_bar);
        let count = vector_embeddings.len();
        h::faiss::populate_vectorestore_index(&index_path, vector_embeddings, PCA_DIMENSIONS)?;
        Ok(count)
    }

    pub(crate) async fn ingest_wikipedia(
        self,
        input_xml: &Path,
        output_directory: &Path,
        ingest_limit: usize,
    ) -> Result<usize, IngestError> {
        match (input_xml.exists(), output_directory.exists()) {
            (true, false) => Err(DirectoryNotFound(output_directory.to_path_buf())),
            (false, _) => Err(XmlNotFound(input_xml.to_path_buf())),
            (true, true) => {
                let markup_db_path = output_directory.join(MARKUP_DB_NAME);
                let markup_pool = h::postgres::get_sqlite_pool(&markup_db_path).await?;

                if !h::postgres::database_is_complete(&markup_pool).await? {
                    log::info!("Preparing markup DB...");
                    self.create_markup_database(&input_xml, &markup_pool, ingest_limit)
                        .await?;
                }
                log::info!("Markup DB is ready at {}", markup_db_path.display());

                let docstore_db_path = output_directory.join(DOCSTORE_DB_NAME);
                let docstore_pool = h::postgres::get_sqlite_pool(&docstore_db_path).await?;

                if !h::postgres::database_is_complete(&docstore_pool).await? {
                    log::info!("Preparing docstore DB...");

                    self.create_docstore_database(markup_pool, &docstore_pool)
                        .await?;
                }
                log::info!("Docstore DB is ready at {}", docstore_db_path.display());

                let tmp_vector_db_path = output_directory.join(VECTOR_TMP_DB_NAME);
                let tmp_vector_pool = h::postgres::get_sqlite_pool(&tmp_vector_db_path).await?;
                if !h::postgres::database_is_complete(&tmp_vector_pool).await? {
                    log::info!("Preparing Vector DB...");

                    self.create_temp_vector_database(docstore_pool, &tmp_vector_pool)
                        .await?;
                }
                log::info!("Vector DB is ready at {}", tmp_vector_db_path.display());

                let index_path = output_directory.join(VECTOR_INDEX_NAME);
                if !h::faiss::index_is_complete(&index_path).map_err(IngestError::IndexError)? {
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
