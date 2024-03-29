use crate::{
    breeder::{
        mutator::{ordering::PopulationOrdering, selector::PopulationSelector},
        prompt::{MutationPrompt, TaskPrompt},
        unit::{Population, ScoredUnit, Unit, UnitData, UnscoredUnit},
        PromptBreedingError,
    },
    openai::{LanguageServiceArguments, LlmMessage, OpenAiDelegate},
};
use simsimd::SpatialSimilarity;
pub(crate) trait GetPopulationPrompt {
    fn get_prompt(&self, population_subsample: &[&ScoredUnit]) -> String;
    fn format_prompt_list(population_subsample: &[&ScoredUnit]) -> String {
        population_subsample
            .iter()
            .enumerate()
            .map(|(index, unit)| format!("{}. {}", index + 1, unit.get_task_prompt()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

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
                .mutate_population(openai, &population_subsample, unit)
                .await?;
            if population_subsample
                .iter()
                .all(|extant_member: &&ScoredUnit| {
                    f32::cosine(new_unit.get_embedding(), extant_member.get_embedding()).unwrap()
                        < 0.95f64
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
                    > 0.95f64
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
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let prompt = self.get_prompt(population_subsample);
        let stop_sequence = format!("{}.", population_subsample.len() + 2);
        let content = openai
            .get_llm_answer(
                LanguageServiceArguments {
                    system: prompt.as_str(),
                    documents: "",
                    query: "",
                    citation_index_begin: 0,
                },
                128u16,
                vec![format!("\n{}", stop_sequence)],
            )
            .await
            .map(|LlmMessage { role: _, content }| content)
            .map_err(PromptBreedingError::LlmError)?;
        let content = content
            .trim()
            .trim_start_matches(stop_sequence.as_str())
            .trim()
            .to_string();
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
