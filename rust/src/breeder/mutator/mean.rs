use crate::{
    breeder::{
        mutator::{
            ordering::PopulationOrdering, population_prompt::GetPopulationPrompt,
            selector::PopulationSelector,
        },
        prompt::{MutationPrompt, TaskPrompt},
        unit::{Population, ScoredUnit, Unit, UnitData, UnscoredUnit},
        PromptBreedingError,
    },
    openai::{LanguageServiceArguments, LlmMessage, OpenAiDelegate},
};
use simsimd::SimSIMD;

impl<T> DistributionEstimationMutator for T where
    T: GetPopulationPrompt + PopulationOrdering + PopulationSelector
{
}
pub(crate) trait DistributionEstimationMutator:
    GetPopulationPrompt + PopulationOrdering + PopulationSelector
{
    async fn mutate(
        &self,
        openai: &OpenAiDelegate,
        population: &Population,
        unit: ScoredUnit,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let mut scored_population = Self::select(population, &unit);

        Self::filter_population(&mut scored_population);
        Self::ordering(&mut scored_population);
        self.create_new_unit(openai, &unit, scored_population).await
    }

    async fn create_new_unit(
        &self,
        openai: &OpenAiDelegate,
        unit: &ScoredUnit,
        population_subsample: Vec<&ScoredUnit>,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        loop {
            let new_unit: UnscoredUnit = self
                .mutate_population(openai, &population_subsample, unit, vec!["\n"])
                .await?;
            if population_subsample
                .iter()
                .all(|extant_member: &&ScoredUnit| {
                    f32::cosine(new_unit.get_embedding(), extant_member.get_embedding()).unwrap()
                        < 0.95f32
                })
            {
                break Ok(new_unit);
            }
        }
    }

    fn filter_population(scored_population: &mut Vec<&ScoredUnit>) {
        let mut i = 0usize;
        let mut len = scored_population.len();
        while i < len {
            if scored_population.iter().all(|extant_member: &&ScoredUnit| {
                f32::cosine(
                    scored_population[i].get_embedding(),
                    extant_member.get_embedding(),
                )
                .unwrap()
                    > 0.95f32
            }) {
                scored_population.remove(i);
                len -= 1;
            } else {
                i += 1;
            }
        }
    }

    async fn mutate_population(
        &self,
        openai: &OpenAiDelegate,
        population_subsample: &[&ScoredUnit],
        unit: &ScoredUnit,
        stop_phrases: Vec<&str>,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let prompt = self.get_prompt(population_subsample);
        let content = openai
            .get_llm_answer(
                LanguageServiceArguments {
                    system: prompt.as_str(),
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
        let content = content.trim().trim_start_matches("1. ").trim().to_string();
        let embedding: Vec<f32> = openai.embed(&content).await.unwrap();
        let task_prompt = TaskPrompt::new(content);
        let new_unit = UnitData {
            problem_description: unit.get_problem_description().clone(),
            task_prompt,
            embedding,
            mutation_instruction: MutationPrompt::new(prompt),
            elites: unit.get_elites().clone(),
            age: unit.get_age() + 1usize,
        };

        Ok(UnscoredUnit { unit: new_unit })
    }
}