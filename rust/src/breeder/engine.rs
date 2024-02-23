use std::sync::Mutex;




use crate::{
    docstore::SqliteDocstore,
    formatter::{CitationStyle, Cite, DocumentFormatter, TextFormatter},
    index::{FaissIndex, SearchService},
    openai::{LanguageServiceServiceArguments},
    openai::{LlmMessage, OpenAiDelegate},
    server::{Source},
};

use super::PromptBreedingError;

pub struct Engine {
    index: Mutex<FaissIndex>,
    openai: OpenAiDelegate,
    docstore: SqliteDocstore,
}

const NUM_DOCUMENTS_TO_RETRIEVE: usize = 4;

const CITATION_STYLE: CitationStyle = CitationStyle::MLA;

impl Engine {
    pub(crate) async fn query(
        &self,
        question: &str,
        system_prompt: &str,
    ) -> Result<String, PromptBreedingError> {
        let (_, _formatted_documents) = self.get_documents(question, 0usize).await?;

        let (_sources, formatted_documents) = self.get_documents(question, 0).await?;

        let llm_service_arguments = LanguageServiceServiceArguments {
            system: system_prompt,
            documents: &formatted_documents,
            query: question,
            citation_index_begin: 0,
        };

        let LlmMessage { role: _, content } = self
            .openai
            .get_llm_answer(llm_service_arguments)
            .await
            .map_err(PromptBreedingError::LlmError)?;

        Ok(content.trim().to_string())
    }

    pub(crate) fn new(
        index: Mutex<FaissIndex>,
        openai: OpenAiDelegate,
        docstore: SqliteDocstore,
    ) -> Self {
        Self {
            index,
            openai,
            docstore,
        }
    }

    pub(crate) async fn get_documents(
        &self,
        user_query: &str,
        num_sources_already_in_chat: usize,
    ) -> Result<(Vec<Source>, String), PromptBreedingError> {
        let embedding = self
            .openai
            .embed(user_query)
            .await
            .map_err(PromptBreedingError::EmbeddingServiceError)?;

        let document_indices = self
            .index
            .lock()
            .map_err(|_| PromptBreedingError::UnableToLockIndex)?
            .search(&embedding, NUM_DOCUMENTS_TO_RETRIEVE)
            .map_err(PromptBreedingError::IndexError)?;

        let documents = self
            .docstore
            .retreive(&document_indices)
            .await
            .map_err(PromptBreedingError::DocstoreError)?;

        let formatted_documents = documents
            .iter()
            .map(|(ordianal, document, provenance)| {
                DocumentFormatter::format_document(
                    *ordianal + num_sources_already_in_chat,
                    &provenance.title(),
                    document,
                )
            })
            .collect::<Vec<String>>()
            .join("\n\n");

        let sources = documents
            .into_iter()
            .zip(document_indices)
            .map(|((ordinal, origin_text, provenance), index)| Source {
                ordinal: ordinal + num_sources_already_in_chat,
                index,
                citation: provenance.format(&CITATION_STYLE),
                url: provenance.url(),
                origin_text,
            })
            .collect::<Vec<_>>();

        Ok((sources, formatted_documents))
    }

    pub(crate) async fn breed_prompt(&self) -> Result<String, PromptBreedingError> {
        todo!()
    }
}
