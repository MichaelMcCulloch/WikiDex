use crate::{
    breeder::{
        prompt::{MutationPrompt, TaskPrompt},
        unit::{ScoredUnit, Unit, UnitData, UnscoredUnit},
        PromptBreedingError,
    },
    openai::{LanguageServiceArguments, LlmMessage, OpenAiDelegate},
};

use super::stop_sequences::StopSequences;
pub(crate) trait PromptForTaskPrompt {
    fn prompt_for_task_prompt(&self, unit: &ScoredUnit) -> String;
}

impl<T> DirectMutator for T where T: PromptForTaskPrompt + StopSequences {}
pub(crate) trait DirectMutator: PromptForTaskPrompt + StopSequences {
    async fn mutate(
        &self,
        openai: &OpenAiDelegate,
        unit: &ScoredUnit,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let prompt = self.prompt_for_task_prompt(unit);
        let content = openai
            .get_llm_answer(
                LanguageServiceArguments {
                    system: prompt.as_str(),
                    documents: "",
                    query: "",
                    citation_index_begin: 0,
                },
                128u16,
                <Self as StopSequences>::stop_sequence(),
            )
            .await
            .map(|LlmMessage { role: _, content }| content)
            .map_err(PromptBreedingError::LlmError)?;
        let content = content.trim().trim_start_matches("1. ").trim().to_string();
        let embedding: Vec<f32> = openai.embed(&content).await.unwrap();
        let task_prompt = TaskPrompt::new(content);
        let new_unit = UnitData {
            problem_description: unit.get_problem_description().clone(),
            task_prompt,
            embedding,
            mutation_prompt: MutationPrompt::new(prompt),
            elites: unit.get_elites().clone(),
            age: unit.get_age() + 1usize,
        };

        Ok(UnscoredUnit { unit: new_unit })
    }
}
