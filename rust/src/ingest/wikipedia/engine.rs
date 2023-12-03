use super::{
    helper::{self as h, text::RecursiveCharacterTextSplitter},
    Ingest,
    IngestError::{self, *},
    WikiMarkupProcessor,
};
use crate::embed::sync::Embedder;
use indicatif::MultiProgress;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::{fs::File, io::BufReader, path::Path, sync::mpsc::channel, thread};

const MARKUP_DB_NAME: &str = "wikipedia_markup.sqlite";
const DOCSTORE_DB_NAME: &str = "wikipedia_docstore.sqlite";
const VECTOR_TMP_DB_NAME: &str = "wikipedia_index.sqlite";
const VECTOR_INDEX_NAME: &str = "wikipedia_index.faiss";

const BATCH_SIZE: usize = 640 * 10;
const PCA_DIMENSIONS: usize = 128;
pub(crate) struct Engine {
    embed: Embedder,
    markup_processor: WikiMarkupProcessor,
    text_splitter: RecursiveCharacterTextSplitter<'static>,
    multi_progress: MultiProgress,
}

impl Engine {
    pub(crate) fn new(
        embed: Embedder,
        multi_progress: MultiProgress,
        chunk_size: usize,
        chunk_overlap: usize,
    ) -> Self {
        let markup_processor = WikiMarkupProcessor::new();

        Self {
            embed,
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

    fn create_markup_database<P: AsRef<Path>>(
        &self,
        input_xml: &P,
        pool: &Pool<SqliteConnectionManager>,
    ) -> Result<usize, IngestError> {
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
            h::sql::populate_markup_db(pool, pages_compressed, access_date, &markup_written_bar)?;

        h::sql::write_completion_timestamp(pool, articles_written)?;
        Ok(article_count)
    }

    fn create_docstore_database(
        &self,
        markup_pool: &Pool<SqliteConnectionManager>,
        docstore_pool: &Pool<SqliteConnectionManager>,
    ) -> Result<usize, IngestError> {
        let document_count = h::sql::count_elements(markup_pool)?.ok_or(NoRows)?;
        //Obtain markup
        let obtain_markup_bar =
            h::progress::new_progress_bar(&self.multi_progress, document_count as u64);
        let pages = h::sql::obtain_markup(markup_pool, &obtain_markup_bar)?;

        let pages_decompressed_bar =
            h::progress::new_progress_bar(&self.multi_progress, pages.len() as u64);
        let documents = h::wiki::decompress_articles_into_documents(
            pages,
            &pages_decompressed_bar,
            &self.markup_processor,
            &self.text_splitter,
        );

        let docstore_written_bar =
            h::progress::new_progress_bar(&self.multi_progress, documents.len() as u64);
        let documents_written: usize =
            h::sql::populate_docstore_db(docstore_pool, documents, &docstore_written_bar)?;
        //process it in parrallel
        h::sql::write_completion_timestamp(docstore_pool, documents_written)?;
        //write it to the database
        Ok(documents_written)
    }
    fn create_temp_vector_database(
        &self,
        docstore_pool: &Pool<SqliteConnectionManager>,
        tmp_vector_pool: &Pool<SqliteConnectionManager>,
    ) -> Result<usize, IngestError> {
        let document_count = h::sql::count_elements(docstore_pool)?.ok_or(NoRows)?;
        let create_vectors_bar =
            h::progress::new_progress_bar(&self.multi_progress, document_count as u64);
        let (tx, rx) = channel::<(Vec<usize>, Vec<Vec<f32>>)>();

        let tmp_vector_pool_clone = tmp_vector_pool.clone();
        let create_vectors_bar_clone = create_vectors_bar.clone();

        create_vectors_bar.set_message("Writing vectorstore to DB...");
        let db_writter_thread = thread::spawn(move || {
            h::sql::write_vectorstore(rx, &tmp_vector_pool_clone, &create_vectors_bar_clone)
        });
        h::sql::populate_vectorstore_db(
            &self.embed,
            docstore_pool,
            document_count,
            tx,
            BATCH_SIZE,
        )?;
        h::sql::write_completion_timestamp(&tmp_vector_pool, document_count)?;
        create_vectors_bar.set_message("Writing vectorstore to DB...DONE");

        let _ = db_writter_thread.join();

        Ok(document_count)
    }

    fn create_vector_index<P: AsRef<Path>>(
        &self,
        tmp_vector_pool: &Pool<SqliteConnectionManager>,
        index_path: &P,
    ) -> Result<usize, IngestError> {
        let vector_count = h::sql::count_elements(tmp_vector_pool)?.ok_or(NoRows)?;
        //Obtain markup
        let obtain_vectors_bar =
            h::progress::new_progress_bar(&self.multi_progress, vector_count as u64);
        let index_path = index_path.as_ref();
        let vector_embeddings = h::sql::obtain_vectors(tmp_vector_pool, &obtain_vectors_bar)?;
        drop(obtain_vectors_bar);
        let count = vector_embeddings.len();
        h::faiss::populate_vectorestore_index(&index_path, vector_embeddings, PCA_DIMENSIONS)?;
        Ok(count)
    }
}

impl Ingest for Engine {
    type E = IngestError;

    fn ingest_wikipedia<P: AsRef<Path>>(
        self,
        input_xml: &P,
        output_directory: &P,
    ) -> Result<usize, Self::E> {
        let input_xml = input_xml.as_ref();
        let output_directory = output_directory.as_ref();

        match (input_xml.exists(), output_directory.exists()) {
            (true, false) => Err(OutputDirectoryNotFound(output_directory.to_path_buf())),
            (false, _) => Err(XmlNotFound(input_xml.to_path_buf())),
            (true, true) => {
                let markup_db_path = output_directory.join(MARKUP_DB_NAME);
                let markup_pool = h::sql::get_sqlite_pool(&markup_db_path).map_err(R2D2Error)?;

                if !h::sql::database_is_complete(&markup_pool)? {
                    log::info!("Preparing markup DB...");
                    self.create_markup_database(&input_xml, &markup_pool)?;
                }
                log::info!("Markup DB is ready at {}", markup_db_path.display());

                let docstore_db_path = output_directory.join(DOCSTORE_DB_NAME);
                let docstore_pool =
                    h::sql::get_sqlite_pool(&docstore_db_path).map_err(R2D2Error)?;

                if !h::sql::database_is_complete(&docstore_pool)? {
                    log::info!("Preparing docstore DB...");

                    self.create_docstore_database(&markup_pool, &docstore_pool)?;
                }
                log::info!("Docstore DB is ready at {}", docstore_db_path.display());
                drop(markup_pool);

                let tmp_vector_db_path = output_directory.join(VECTOR_TMP_DB_NAME);
                let tmp_vector_pool =
                    h::sql::get_sqlite_pool(&tmp_vector_db_path).map_err(R2D2Error)?;
                if !h::sql::database_is_complete(&tmp_vector_pool)? {
                    log::info!("Preparing Vector DB...");

                    self.create_temp_vector_database(&docstore_pool, &tmp_vector_pool)?;
                }
                log::info!("Vector DB is ready at {}", tmp_vector_db_path.display());

                drop(docstore_pool);
                let index_path = output_directory.join(VECTOR_INDEX_NAME);
                if !h::faiss::index_is_complete(&index_path).map_err(IndexError)? {
                    log::info!("Preparing Vector Index...");

                    self.create_vector_index(&tmp_vector_pool, &index_path)?;
                }
                log::info!("Vector Index is ready at {}", index_path.display());

                Ok(1)
            }
        }
    }
}
