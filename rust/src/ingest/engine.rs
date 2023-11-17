use super::{
    helper as h, Ingest,
    IngestError::{self, *},
};
use crate::{embed::Embedder, llm::OpenAiService};
use indicatif::MultiProgress;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use std::{fs::File, io::BufReader, path::Path};

const MARKUP_DB_NAME: &str = "wikipedia_markup.sqlite";
const DOCSTORE_DB_NAME: &str = "wikipedia_docstore.sqlite";
const VECTOR_DB_NAME: &str = "wikipedia_index.faiss";

pub(crate) struct Engine {
    embed: Embedder,
    llm: OpenAiService,
    thread_count: usize,
    multi_progress: MultiProgress,
}

impl Engine {
    pub(crate) fn new(embed: Embedder, llm: OpenAiService, multi_progress: MultiProgress) -> Self {
        Self {
            embed,
            llm,
            thread_count: 32,
            multi_progress,
        }
    }

    fn create_markup_database<P: AsRef<Path>>(
        &self,
        input_xml: &P,
        connection: &PooledConnection<SqliteConnectionManager>,
    ) -> Result<usize, <Self as Ingest>::E> {
        let access_date = h::wiki::get_date_from_xml_name(input_xml)?;
        let file = BufReader::with_capacity(
            2 * 1024 * 1024,
            File::open(input_xml.as_ref()).map_err(IoError)?,
        );

        let eligible_pages_bar = h::pb::new_progress_bar(&self.multi_progress, 7000000);
        let eligible_pages = h::wiki::get_eligible_pages(file, &eligible_pages_bar);
        let pages_compressed_bar =
            h::pb::new_progress_bar(&self.multi_progress, eligible_pages.len() as u64);
        let pages_compressed = h::wiki::compress_articles(eligible_pages, &pages_compressed_bar);
        let article_count = pages_compressed.len();
        let markup_written_bar =
            h::pb::new_progress_bar(&self.multi_progress, article_count as u64);
        h::sql::populate_markup_db(
            connection,
            pages_compressed,
            access_date,
            &markup_written_bar,
        )?;

        h::sql::write_completion_timestamp(connection, article_count)?;
        Ok(article_count)
    }

    fn create_docstore_database<P: AsRef<Path>>(
        &self,
        input_xml: &P,
        connection: &PooledConnection<SqliteConnectionManager>,
    ) -> Result<usize, <Self as Ingest>::E> {
        let access_date = h::wiki::get_date_from_xml_name(input_xml)?;
        let file = BufReader::with_capacity(
            2 * 1024 * 1024,
            File::open(input_xml.as_ref()).map_err(IoError)?,
        );

        let eligible_pages_bar = h::pb::new_progress_bar(&self.multi_progress, 7000000);
        let eligible_pages = h::wiki::get_eligible_pages(file, &eligible_pages_bar);
        let pages_compressed_bar =
            h::pb::new_progress_bar(&self.multi_progress, eligible_pages.len() as u64);
        let pages_compressed = h::wiki::compress_articles(eligible_pages, &pages_compressed_bar);
        let article_count = pages_compressed.len();
        let markup_written_bar =
            h::pb::new_progress_bar(&self.multi_progress, article_count as u64);
        h::sql::populate_markup_db(
            connection,
            pages_compressed,
            access_date,
            &markup_written_bar,
        )?;

        h::sql::write_completion_timestamp(connection, article_count)?;
        Ok(article_count)
    }
}

impl Ingest for Engine {
    type E = IngestError;

    fn ingest_wikipedia<P: AsRef<Path>>(
        self,
        input_xml: &P,
        output_directory: &P,
    ) -> Result<usize, Self::E> {
        match (
            input_xml.as_ref().exists(),
            output_directory.as_ref().exists(),
        ) {
            (true, false) => Err(OutputDirectoryNotFound(
                output_directory.as_ref().to_path_buf(),
            )),
            (false, _) => Err(XmlNotFound(input_xml.as_ref().to_path_buf())),
            (true, true) => {
                let markup_db_path = output_directory.as_ref().join(MARKUP_DB_NAME);
                let markup_connection = h::sql::get_sqlite_pool(&markup_db_path)
                    .and_then(|pool| pool.get())
                    .map_err(R2D2Error)?;

                if !h::sql::database_is_complete(&markup_connection)? {
                    log::info!("Preparing Markup DB...");
                    self.create_markup_database(input_xml, &markup_connection)?;
                }
                log::info!("Markup DB is ready at {}", markup_db_path.display());

                let docstore_db_path = output_directory.as_ref().join(DOCSTORE_DB_NAME);
                let docstore_connection = h::sql::get_sqlite_pool(&docstore_db_path)
                    .and_then(|pool| pool.get())
                    .map_err(R2D2Error)?;

                if !h::sql::database_is_complete(&docstore_connection)? {
                    log::info!("Preparing docstore DB...");
                    self.create_docstore_database(input_xml, &docstore_connection)?;
                }
                log::info!("docstore DB is ready at {}", docstore_db_path.display());

                Ok(1)
            }
        }
    }
}
