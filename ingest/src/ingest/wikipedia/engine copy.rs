use super::{
    helper::{self as h, text::RecursiveCharacterTextSplitter},
    IngestError::{self, *},
    WikiMarkupProcessor,
};
use crate::openai::OpenAiDelegate;

use indicatif::MultiProgress;
use sqlx::PgPool;
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tokio::sync::mpsc::unbounded_channel;

const MARKUP_DB_NAME: &str = "wikipedia_markup.sqlite";
const DOCSTORE_DB_NAME: &str = "wikipedia_docstore.sqlite";
const VECTOR_TMP_DB_NAME: &str = "wikipedia_index.sqlite";
const VECTOR_INDEX_NAME: &str = "wikipedia_index.faiss";

const PCA_DIMENSIONS: usize = 128;
const MINIMUM_PASSAGE_LENGTH_IN_WORDS: usize = 15;
