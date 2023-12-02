use super::{
    helper::{self as h, text::RecursiveCharacterTextSplitter},
    Ingest,
    IngestError::{self, *},
    WikiMarkupProcessor,
};
use crate::{embed::Embedder, llm::SyncOpenAiService};
use indicatif::MultiProgress;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{fs::File, io::BufReader, path::Path};

const MARKUP_DB_NAME: &str = "wikipedia_markup.sqlite";
const DOCSTORE_DB_NAME: &str = "wikipedia_docstore.sqlite";
const VECTOR_DB_NAME: &str = "wikipedia_index.faiss";

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
    ) -> Result<usize, <Self as Ingest>::E> {
        //Obtain markup
        let obtain_markup_bar = h::progress::new_progress_bar(&self.multi_progress, 7000000 as u64);
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
        let documents_written =
            h::sql::populate_docstore_db(docstore_pool, documents, &docstore_written_bar)?;
        //process it in parrallel
        h::sql::write_completion_timestamp(docstore_pool, documents_written)?;
        //write it to the database
        Ok(documents_written)
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

                Ok(1)
            }
        }
    }
}
