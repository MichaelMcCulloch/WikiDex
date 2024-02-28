use super::{
    unit::{Population, ScoredUnit, Unit, UnscoredUnit},
    Mutator, PromptBreedingError,
};
use crate::{
    breeder::prompt::{MutationPrompt, ThinkingStyle},
    openai::OpenAiDelegate,
};
use rand::seq::SliceRandom;
use simsimd::SimSIMD;

pub(crate) struct ZeroOrderPromptGeneration {}
pub(crate) struct FirstOrderPromptGeneration {}
pub(crate) struct EstimationOfDistributionMutation {}
pub(crate) struct RankAndIndexMutation {}

impl Mutator for ZeroOrderPromptGeneration {}
impl Mutator for FirstOrderPromptGeneration {}
impl Mutator for EstimationOfDistributionMutation {}
impl Mutator for RankAndIndexMutation {}

impl ZeroOrderPromptGeneration {
    pub(crate) async fn mutate(
        openai: &OpenAiDelegate,
        unit: ScoredUnit,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let mutation_instruction = MutationPrompt::new(format!(
            "{} A list of 100 hints:\n1. ",
            unit.get_problem_description()
        ));
        <Self as Mutator>::mutate(openai, mutation_instruction, &unit, 0, vec!["\n2"]).await
    }
}
impl FirstOrderPromptGeneration {
    pub(crate) async fn mutate(
        openai: &OpenAiDelegate,
        unit: ScoredUnit,
        mutation_directive: MutationPrompt,
        thinking_style: ThinkingStyle,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let mutation_instruction = MutationPrompt::new(format!(
            "MUTATION: {mutation_directive} {thinking_style}\nINSTRUCTION: {}\nINSTRUCTION MUTANT:",
            unit.get_task_prompt()
        ));
        <Self as Mutator>::mutate(openai, mutation_instruction, &unit, 0, vec!["\n"]).await
    }
}
impl EstimationOfDistributionMutation {
    pub(crate) async fn mutate(
        openai: &OpenAiDelegate,
        population: &Population,
        unit: ScoredUnit,
        _mutation_directive: MutationPrompt,
        _thinking_style: ThinkingStyle,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let mut population_subsample: Vec<&ScoredUnit> = vec![];

        for member in &population.scored {
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
        let len = population_subsample.len();
        population_subsample.shuffle(&mut rand::thread_rng());
        let prompt_list = population_subsample
            .iter()
            .enumerate()
            .map(|(index, unit)| format!("{}. {}", index + 1, unit.get_task_prompt()))
            .collect::<Vec<_>>()
            .join("\n");

        loop {
            let mutation_instruction = MutationPrompt::new(format!(
                "Continue the series with more items:\n{prompt_list}\n{}.",
                len + 1
            ));
            let new_unit =
                <Self as Mutator>::mutate(openai, mutation_instruction, &unit, 0, vec!["\n"])
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
}

impl RankAndIndexMutation {
    pub(crate) async fn mutate(
        openai: &OpenAiDelegate,
        population: &Population,
        unit: ScoredUnit,
        _mutation_directive: MutationPrompt,
        _thinking_style: ThinkingStyle,
    ) -> Result<UnscoredUnit, PromptBreedingError> {
        let mut population_subsample: Vec<&ScoredUnit> = vec![];

        for member in &population.scored {
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
        let len = population_subsample.len();
        population_subsample.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());
        let prompt_list = population_subsample
            .iter()
            .enumerate()
            .map(|(index, unit)| format!("{}. {}", index + 1, unit.get_task_prompt()))
            .collect::<Vec<_>>()
            .join("\n");

        loop {
            let mutation_instruction = MutationPrompt::new(format!(
                "Continue the series with more items:\n{prompt_list}\n{}.",
                len + 1
            ));
            let new_unit =
                <Self as Mutator>::mutate(openai, mutation_instruction, &unit, 0, vec!["\n"])
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
}
