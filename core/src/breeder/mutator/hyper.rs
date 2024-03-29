use crate::{
    breeder::{
        prompt::{MutationPrompt, TaskPrompt},
        unit::{ScoredUnit, Unit, UnitData, UnscoredUnit},
        PromptBreedingError,
    },
    openai::{LanguageServiceArguments, LlmMessage, OpenAiDelegate},
};

pub(crate) trait PromptForMutatorPrompt {
    fn prompt_for_meta_prompt(&self, unit: &ScoredUnit) -> String;
}

impl<T> MetaMutator for T where T: PromptForMutatorPrompt {}
pub(crate) trait MetaMutator: PromptForMutatorPrompt {
    async fn mutate(
        &self,
        openai: &OpenAiDelegate,
        unit: &ScoredUnit,
        stop_phrases: Vec<&str>,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let prompt = self.prompt_for_meta_prompt(unit);

        let mutator_prompt_content = openai
            .get_llm_answer(
                LanguageServiceArguments {
                    system: prompt.as_str(),
                    documents: "",
                    query: "",
                    citation_index_begin: 0,
                },
                128u16,
                stop_phrases.clone(),
            )
            .await
            .map(|LlmMessage { role: _, content }| content)
            .map_err(PromptBreedingError::LlmError)?;
        let mutator_prompt_content = mutator_prompt_content
            .trim()
            .trim_start_matches("1. ")
            .trim()
            .to_string();

        let task_prompt_prompt = format!(
            "MUTATION: {}\nINSTRUCTION: {}\nINSTRUCTION MUTANT:",
            mutator_prompt_content,
            unit.get_task_prompt()
        );
        let task_prompt_content = openai
            .get_llm_answer(
                LanguageServiceArguments {
                    system: task_prompt_prompt.as_str(),
                    documents: "",
                    query: "",
                    citation_index_begin: 0,
                },
                128u16,
                stop_phrases,
            )
            .await
            .map(|LlmMessage { role: _, content }| content)
            .map_err(PromptBreedingError::LlmError)?;
        let task_prompt_content = task_prompt_content
            .trim()
            .trim_start_matches("1. ")
            .trim()
            .to_string();

        let embedding: Vec<f32> = openai.embed(&task_prompt_content).await.unwrap();
        let task_prompt = TaskPrompt::new(task_prompt_content);
        let new_unit = UnitData {
            problem_description: unit.get_problem_description().clone(),
            task_prompt,
            embedding,
            mutation_prompt: MutationPrompt::new(task_prompt_prompt),
            elites: unit.get_elites().clone(),
            age: unit.get_age() + 1usize,
        };

        Ok(UnscoredUnit { unit: new_unit })
    }
}
