#[cfg(feature = "postgres")]
mod posgres;
#[cfg(feature = "sqlite")]
mod sqlite;

use super::{helper::text::RecursiveCharacterTextSplitter, WikiMarkupProcessor};
use crate::openai::OpenAiDelegate;

use indicatif::MultiProgress;
use sqlx::Database;
use std::{marker::PhantomData, sync::Arc};

const MARKUP_DB_NAME: &str = "wikipedia_markup.sqlite";
const DOCSTORE_DB_NAME: &str = "wikipedia_docstore.sqlite";
const VECTOR_TMP_DB_NAME: &str = "wikipedia_index.sqlite";
const VECTOR_INDEX_NAME: &str = "wikipedia_index.faiss";

const PCA_DIMENSIONS: usize = 128;
const MINIMUM_PASSAGE_LENGTH_IN_WORDS: usize = 15;

pub(crate) struct Engine<P: Database> {
    openai: Arc<OpenAiDelegate>,
    markup_processor: WikiMarkupProcessor,
    text_splitter: RecursiveCharacterTextSplitter<'static>,
    multi_progress: MultiProgress,
    _phantom: PhantomData<P>,
}
