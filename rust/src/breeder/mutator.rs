use super::{
    prompt::{MutationPrompt, TaskPrompt},
    unit::{ScoredUnit, Unit, UnitData, UnscoredUnit},
    PromptBreedingError,
};
use crate::openai::{LanguageServiceArguments, LlmMessage, OpenAiDelegate};

pub(crate) trait Mutator {
    async fn mutate(
        openai: &OpenAiDelegate,
        mutation_instruction: MutationPrompt,
        unit: &ScoredUnit,
        citation_index_begin: usize,
        stop_phrases: Vec<&str>,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let content = openai
            .get_llm_answer(
                LanguageServiceArguments {
                    system: format!("{mutation_instruction}").as_str(),
                    documents: "",
                    query: "",
                    citation_index_begin,
                },
                128u16,
                stop_phrases,
            )
            .await
            .map(|LlmMessage { role: _, content }| content)
            .map_err(PromptBreedingError::LlmError)?;
        let embedding = openai.embed(&content).await.unwrap();
        let task_prompt = TaskPrompt::new(content);
        let new_unit = UnitData {
            problem_description: unit.get_problem_description().clone(),
            task_prompt,
            embedding,
            mutation_instruction,
            elites: unit.get_elites().clone(),
        };

        Ok(UnscoredUnit { unit: new_unit })
    }
}
