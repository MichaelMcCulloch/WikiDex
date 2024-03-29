use rand::seq::SliceRandom;

use crate::breeder::{
    mutator::{
        mean::GetPopulationPrompt, ordering::PopulationOrdering, selector::PopulationSelector,
    },
    unit::{Population, ScoredUnit, Unit},
};
pub(crate) struct EstimationOfDistributionMutation {}
impl PopulationSelector for EstimationOfDistributionMutation {
    fn select<'a>(population: &'a Population, _unit: &'a ScoredUnit) -> Vec<&'a ScoredUnit> {
        population.scored.iter().collect::<Vec<_>>()
    }
}
impl PopulationOrdering for EstimationOfDistributionMutation {
    fn ordering(population_subsample: &mut Vec<&ScoredUnit>) {
        population_subsample.shuffle(&mut rand::thread_rng())
    }
}
impl GetPopulationPrompt for EstimationOfDistributionMutation {
    fn get_prompt(&self, population_subsample: &[&ScoredUnit]) -> String {
        let len = population_subsample.len();
        let prompt_list = Self::format_prompt_list(population_subsample);
        format!(
            "A List of responses in random order of score.\n{prompt_list}\n{}.",
            len + 1
        )
    }
}

pub(crate) struct RankAndIndexMutation {}
impl PopulationSelector for RankAndIndexMutation {
    fn select<'a>(population: &'a Population, _unit: &'a ScoredUnit) -> Vec<&'a ScoredUnit> {
        population.scored.iter().collect::<Vec<_>>()
    }
}

impl PopulationOrdering for RankAndIndexMutation {
    fn ordering(population_subsample: &mut Vec<&ScoredUnit>) {
        population_subsample.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap())
    }
}
impl GetPopulationPrompt for RankAndIndexMutation {
    fn get_prompt(&self, population_subsample: &[&ScoredUnit]) -> String {
        let len = population_subsample.len();
        let prompt_list = Self::format_prompt_list(population_subsample);
        format!(
            "A List of responses in descending order of score.\n{prompt_list}\n{}.",
            len + 1
        )
    }
}

pub(crate) struct LineageMutation {}
impl PopulationSelector for LineageMutation {
    fn select<'a>(_population: &'a Population, unit: &'a ScoredUnit) -> Vec<&'a ScoredUnit> {
        unit.get_elites().iter().collect::<Vec<_>>()
    }
}

impl PopulationOrdering for LineageMutation {
    fn ordering(population_subsample: &mut Vec<&ScoredUnit>) {
        population_subsample.sort_by(|a, b| a.get_age().partial_cmp(b.get_age()).unwrap())
    }
}
impl GetPopulationPrompt for LineageMutation {
    fn get_prompt(&self, population_subsample: &[&ScoredUnit]) -> String {
        let len = population_subsample.len();
        let prompt_list = Self::format_prompt_list(population_subsample);

        format!(
            "Instruction variants found in ascending order of quality:\n{prompt_list}\n{}.",
            len + 1
        )
    }
}

#[cfg(test)]
mod test {

    use super::{EstimationOfDistributionMutation, LineageMutation, RankAndIndexMutation};
    use crate::{
        breeder::{
            mutator::mean::DistributionEstimationMutator,
            prompt::{MutationPrompt, ProblemDescription, TaskPrompt},
            unit::{Population, ScoredUnit, UnitData},
        },
        openai::{OpenAiDelegate, OpenAiDelegateBuilder, OpenAiDelegateBuilderArgument},
    };
    use url::Url;

    const PROBLEM_DESCRIPTION: &str = "Pour water out of a boot.";
    const PROBLEM_DESCRIPTION_2: &str = "Evacuate the moisture from footwear.";
    const PROBLEM_DESCRIPTION_3: &str = "Dry the sandals.";

    async fn obtain_task_prompt(
        openai: &OpenAiDelegate,
        problem_description: &str,
    ) -> (TaskPrompt, Vec<f32>) {
        let embedding = openai.embed(problem_description).await.unwrap();
        (TaskPrompt::new(problem_description), embedding)
    }

    async fn obtain_unit_data(
        openai: &OpenAiDelegate,
        problem_description: &str,
        elites: Vec<ScoredUnit>,
    ) -> UnitData {
        let task_prompt_and_embedding = obtain_task_prompt(openai, problem_description).await;
        let mutation_prompt = MutationPrompt::new(problem_description);
        let problem_description = ProblemDescription::new(problem_description);
        let task_prompt = task_prompt_and_embedding.0;
        let embedding = task_prompt_and_embedding.1;
        UnitData {
            problem_description,
            task_prompt,
            embedding,
            mutation_prompt,
            elites,
            age: 0,
        }
    }

    async fn obtain_scored_unit(
        openai: &OpenAiDelegate,
        problem_description: &str,
        score: f32,
        elites: Vec<ScoredUnit>,
    ) -> ScoredUnit {
        ScoredUnit {
            unit: obtain_unit_data(openai, problem_description, elites).await,
            fitness: score,
        }
    }

    fn obtain_openai() -> OpenAiDelegate {
        let openai_builder =
            OpenAiDelegateBuilder::with_embedding(OpenAiDelegateBuilderArgument::Endpoint(
                Url::parse("https://infinity.semanticallyinvalid.net/").unwrap(),
                Some(String::from("")),
                String::from("thenlper/gte-small"),
            ));

        openai_builder.with_instruct(OpenAiDelegateBuilderArgument::Endpoint(
            Url::parse("https://vllm.semanticallyinvalid.net/v1/").unwrap(),
            Some(String::from("")),
            String::from("TheBloke/Mistral-7B-Instruct-v0.2-AWQ"),
        ))
    }

    #[tokio::test]
    async fn EstimationOfDistributionMutation() {
        let openai = obtain_openai();

        let scored_members = vec![
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32, vec![]).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_2, 0.02f32, vec![]).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_3, 0.03f32, vec![]).await,
        ];

        let scored_members = vec![
            obtain_scored_unit(
                &openai,
                PROBLEM_DESCRIPTION,
                0.01f32,
                scored_members.clone(),
            )
            .await,
            obtain_scored_unit(
                &openai,
                PROBLEM_DESCRIPTION_2,
                0.02f32,
                scored_members.clone(),
            )
            .await,
            obtain_scored_unit(
                &openai,
                PROBLEM_DESCRIPTION_3,
                0.03f32,
                scored_members.clone(),
            )
            .await,
        ];

        let population = Population {
            unscored: vec![],
            scored: scored_members.clone(),
        };

        let operator = EstimationOfDistributionMutation {};
        let new_unit = operator
            .mutate(&openai, &population, scored_members.get(2).unwrap().clone())
            .await;

        match new_unit {
            Ok(mutant) => {
                println!("{mutant}");
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }
    #[tokio::test]
    async fn RankAndIndexMutation() {
        let openai = obtain_openai();

        let scored_members = vec![
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32, vec![]).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_2, 0.02f32, vec![]).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_3, 0.03f32, vec![]).await,
        ];

        let scored_members = vec![
            obtain_scored_unit(
                &openai,
                PROBLEM_DESCRIPTION,
                0.01f32,
                scored_members.clone(),
            )
            .await,
            obtain_scored_unit(
                &openai,
                PROBLEM_DESCRIPTION_2,
                0.02f32,
                scored_members.clone(),
            )
            .await,
            obtain_scored_unit(
                &openai,
                PROBLEM_DESCRIPTION_3,
                0.03f32,
                scored_members.clone(),
            )
            .await,
        ];

        let population = Population {
            unscored: vec![],
            scored: scored_members.clone(),
        };

        let operator = RankAndIndexMutation {};
        let new_unit = operator
            .mutate(&openai, &population, scored_members.get(2).unwrap().clone())
            .await;

        match new_unit {
            Ok(mutant) => {
                println!("{mutant}");
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }

    #[tokio::test]
    async fn LineageBasedMutation() {
        let openai = obtain_openai();

        let scored_members = vec![
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32, vec![]).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_2, 0.02f32, vec![]).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_3, 0.03f32, vec![]).await,
        ];

        let scored_members = vec![
            obtain_scored_unit(
                &openai,
                PROBLEM_DESCRIPTION,
                0.01f32,
                scored_members.clone(),
            )
            .await,
            obtain_scored_unit(
                &openai,
                PROBLEM_DESCRIPTION_2,
                0.02f32,
                scored_members.clone(),
            )
            .await,
            obtain_scored_unit(
                &openai,
                PROBLEM_DESCRIPTION_3,
                0.03f32,
                scored_members.clone(),
            )
            .await,
        ];

        let population = Population {
            unscored: vec![],
            scored: scored_members.clone(),
        };

        let operator = LineageMutation {};
        let new_unit = operator
            .mutate(&openai, &population, scored_members.get(2).unwrap().clone())
            .await;

        match new_unit {
            Ok(mutant) => {
                println!("{mutant}");
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }
}
