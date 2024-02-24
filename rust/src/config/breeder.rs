use crate::{cli_args::BreederArgs, openai::ModelKind};
use colored::Colorize;
use std::{fmt::Display, path::PathBuf};
use url::Url;
#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) index: PathBuf,
    pub(crate) docstore: PathBuf,
    pub(crate) thinking_styles_db: PathBuf,
    pub(crate) mutation_prompts_db: PathBuf,
    pub(crate) output_directory: PathBuf,
    pub(crate) embed_url: Url,
    pub(crate) embed_model_name: PathBuf,
    pub(crate) llm_url: Url,
    pub(crate) language_model_name: PathBuf,
    pub(crate) language_model_kind: ModelKind,
    pub(crate) generation_limit: usize,
}

impl From<BreederArgs> for Config {
    fn from(value: BreederArgs) -> Self {
        Config {
            output_directory: value.output_directory,
            embed_url: value.embed_url,
            llm_url: value.llm_url,
            language_model_name: value.language_model_name,
            language_model_kind: value.language_model_kind,
            embed_model_name: value.embed_model_name,
            generation_limit: value.generation_limit,
            index: value.index,
            docstore: value.docstore,
            thinking_styles_db: value.thinking_styles_db,
            mutation_prompts_db: value.mutation_prompts_db,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Config {
            output_directory,
            embed_url,
            ..
        } = self;

        let output_directory = output_directory.display();
        let _embed_url = embed_url.as_str().yellow();

        write!(
            f,
            "Breeder running.\n\t.\n\tWriting output at {output_directory}.",
        )
    }
}
