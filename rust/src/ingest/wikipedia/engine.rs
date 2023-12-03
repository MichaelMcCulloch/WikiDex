use super::{
    helper::{
        self as h,
        gzip_helper::decompress_text,
        sql::{
            init_temp_embedding_sqlite_pool, DOCSTORE_DB_DOCUMENT_TABLE_NAME,
            MARKUP_DB_WIKI_MARKUP_TABLE_NAME,
        },
        text::RecursiveCharacterTextSplitter,
    },
    Ingest,
    IngestError::{self, *},
    WikiMarkupProcessor,
};
use crate::{
    embed::{sync::Embedder, EmbedServiceSync},
    ingest::wikipedia::helper::sql::EMBEDDINGS_DB_EMBEDDINGS_TABLE_NAME,
};
use indicatif::MultiProgress;
use r2d2::Pool;
use r2d2_sqlite::{rusqlite::params, SqliteConnectionManager};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    fs::File,
    io::{BufReader, Write},
    path::Path,
    sync::mpsc::channel,
    thread,
};

const MARKUP_DB_NAME: &str = "wikipedia_markup.sqlite";
const DOCSTORE_DB_NAME: &str = "wikipedia_docstore.sqlite";
const VECTOR_TMP_DB_NAME: &str = "wikipedia_index.sqlite";
const VECTOR_DB_NAME: &str = "wikipedia_index.faiss";

const BATCH_SIZE: usize = 640 * 10;
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
    ) -> Result<usize, <Self as Ingest>::E> {
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
    ) -> Result<usize, <Self as Ingest>::E> {
        let document_count = h::sql::count_elements(docstore_pool)?.ok_or(NoRows)?;
        let create_vectors_bar =
            h::progress::new_progress_bar(&self.multi_progress, document_count as u64);
        let (tx, rx) = channel::<(Vec<usize>, Vec<Vec<f32>>)>();

        let tmp_vector_pool_clone = tmp_vector_pool.clone();
        let create_vectors_bar_clone = create_vectors_bar.clone();

        let db_writter_thread = thread::spawn(move || {
            let tmp_vector_connection = &tmp_vector_pool_clone.get().map_err(R2D2Error)?;
            init_temp_embedding_sqlite_pool(&tmp_vector_connection)?;

            while let Ok((indices, embeddings)) = rx.recv() {
                tmp_vector_connection
                    .execute_batch("BEGIN;")
                    .map_err(RuSqliteError)?;
                let count = indices.len();
                for (index, embedding) in indices.into_iter().zip(embeddings) {
                    let mut v8: Vec<u8> = vec![];

                    for e in embedding {
                        v8.write_all(&e.to_le_bytes()).map_err(IoError)?;
                    }

                    tmp_vector_connection
                            .execute(
                                &format!("INSERT INTO {EMBEDDINGS_DB_EMBEDDINGS_TABLE_NAME} (id, gte_small) VALUES ($1, $2)"),
                                params![index, v8],
                            )
                            .map_err(RuSqliteError)?;
                }
                create_vectors_bar_clone.inc(count as u64);

                tmp_vector_connection
                    .execute_batch("COMMIT;")
                    .map_err(RuSqliteError)?;
            }
            Ok::<(), IngestError>(())
        });
        create_vectors_bar.set_message("Writing Vectorstore to DB...");
        let docstore_connection = &docstore_pool.get().map_err(R2D2Error)?;
        for indices in (0..document_count).collect::<Vec<_>>().chunks(BATCH_SIZE) {
            let mut stmt_read_document = docstore_connection
                .prepare(&format!(
                    "SELECT id, text FROM {DOCSTORE_DB_DOCUMENT_TABLE_NAME} WHERE id >= $1 AND id <= $2 ORDER BY id ASC;"
                ))
                .map_err(RuSqliteError)?;

            let start = indices.first().unwrap();
            let end = indices.last().unwrap();

            let mapped = stmt_read_document
                .query_map(params![start, end], |row| {
                    let id: usize = row.get(0)?;
                    let doc: Vec<u8> = row.get(1)?;
                    Ok((id, doc))
                })
                .map_err(RuSqliteError)?
                .filter_map(|f| f.ok())
                .collect::<Vec<_>>();

            let rows = mapped
                .into_par_iter()
                .filter_map(|(id, doc)| Some((id, decompress_text(doc).ok()?)))
                .collect::<Vec<_>>();

            let (ids, batch): (Vec<usize>, Vec<String>) = rows.into_iter().unzip();
            let batch = batch.iter().map(|s| s.as_str()).collect::<Vec<_>>();

            let batch_result = self.embed.embed(&batch).map_err(EmbeddingServiceError)?;
            let _ = tx.send((ids, batch_result));
        }
        drop(tx);
        db_writter_thread.join().unwrap()?;
        h::sql::write_completion_timestamp(&tmp_vector_pool, document_count)?;

        create_vectors_bar.set_message("Writing Vectorstore to DB...DONE");
        Ok(0)
    }
}

impl Ingest for Engine {
    type E = IngestError;

    fn ingest_wikipedia(self, input_xml: &Path, output_directory: &Path) -> Result<usize, Self::E> {
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
                log::info!("Vector DB is ready at {}", docstore_db_path.display());

                Ok(1)
            }
        }
    }
}
