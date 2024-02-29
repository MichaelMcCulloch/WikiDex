use super::{
    prompt::{MutationPrompt, TaskPrompt},
    unit::{Population, ScoredUnit, Unit, UnitData, UnscoredUnit},
    PromptBreedingError,
};
use crate::openai::{LanguageServiceArguments, LlmMessage, OpenAiDelegate};
use simsimd::SimSIMD;

pub(crate) trait GetUnitPrompt {
    fn get_prompt(&self, unit: &ScoredUnit) -> String;
}
pub(crate) trait GetDistributionPrompt {
    fn get_prompt(&self, population_subsample: &[&ScoredUnit]) -> String;
}

pub(crate) trait DirectMutator: GetUnitPrompt {
    async fn mutate(
        &self,
        openai: &OpenAiDelegate,
        unit: &ScoredUnit,
        stop_phrases: Vec<&str>,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let prompt = self.get_prompt(unit);
        println!("{prompt}");

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
        };

        Ok(UnscoredUnit { unit: new_unit })
    }
}
pub(crate) trait EstimateDistributionMutator: GetDistributionPrompt {
    fn ordering(population_subsample: &mut Vec<&ScoredUnit>);
    async fn mutate(
        &self,
        openai: &OpenAiDelegate,
        population: &Population,
        unit: ScoredUnit,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let scored_population = &population.scored;
        let mut population_subsample: Vec<&ScoredUnit> = vec![];
        for member in scored_population {
            if population_subsample
                .iter()
                .all(|extant_member: &&ScoredUnit| {
                    f32::cosine(member.get_embedding(), extant_member.get_embedding()).unwrap()
                        < 0.95f32
                })
            {
                population_subsample.push(member);
            }
        }

        population_subsample.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        self.select_member(openai, unit, population_subsample).await
    }

    fn format_prompt_list(population_subsample: &[&ScoredUnit]) -> String {
        population_subsample
            .iter()
            .enumerate()
            .map(|(index, unit)| format!("{}. {}", index + 1, unit.get_task_prompt()))
            .collect::<Vec<_>>()
            .join("\n")
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
        };

        Ok(UnscoredUnit { unit: new_unit })
    }
    async fn select_member(
        &self,
        openai: &OpenAiDelegate,
        unit: ScoredUnit,
        population_subsample: Vec<&ScoredUnit>,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        loop {
            let new_unit: UnscoredUnit = self
                .mutate_population(openai, &population_subsample, &unit, vec!["\n"])
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
    fn filter_population(scored_population: &Vec<ScoredUnit>) -> Vec<&ScoredUnit> {
        let mut population_subsample: Vec<&ScoredUnit> = vec![];
        for member in scored_population {
            if population_subsample
                .iter()
                .all(|extant_member: &&ScoredUnit| {
                    f32::cosine(member.get_embedding(), extant_member.get_embedding()).unwrap()
                        < 0.95f32
                })
            {
                population_subsample.push(member);
            }
        }
        population_subsample
    }
}
