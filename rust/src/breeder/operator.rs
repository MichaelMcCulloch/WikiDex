use super::{
    unit::{Population, ScoredUnit, Unit},
    DirectMutator, GetPopulationPrompt, GetUnitPrompt, PopulationOrdering, PopulationSelector,
};
use crate::breeder::prompt::{MutationPrompt, ThinkingStyle};
use rand::seq::SliceRandom;

pub(crate) struct ZeroOrderPromptGeneration {}
pub(crate) struct FirstOrderPromptGeneration {
    pub(crate) mutation_prompt: MutationPrompt,
    pub(crate) thinking_style: ThinkingStyle,
}
pub(crate) struct EstimationOfDistributionMutation {}
pub(crate) struct RankAndIndexMutation {}
pub(crate) struct LineageMutation {}

impl DirectMutator for ZeroOrderPromptGeneration {}
impl DirectMutator for FirstOrderPromptGeneration {}

impl GetUnitPrompt for ZeroOrderPromptGeneration {
    fn get_prompt(&self, unit: &ScoredUnit) -> String {
        format!(
            "INSTRUCTION: {} A list of 100 hints:\n1. ",
            unit.get_problem_description()
        )
    }
}
impl GetUnitPrompt for FirstOrderPromptGeneration {
    fn get_prompt(&self, unit: &ScoredUnit) -> String {
        format!(
            "MUTATION: {} {}\nINSTRUCTION: {}\nINSTRUCTION MUTANT:",
            self.mutation_prompt,
            self.thinking_style,
            unit.get_task_prompt()
        )
    }
}
impl PopulationSelector for EstimationOfDistributionMutation {
    fn select(population: &Population) -> Vec<&ScoredUnit> {
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
impl PopulationSelector for RankAndIndexMutation {
    fn select(population: &Population) -> Vec<&ScoredUnit> {
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

impl PopulationSelector for LineageMutation {
    fn select(population: &Population) -> Vec<&ScoredUnit> {
        population.elites.iter().collect::<Vec<_>>()
    }
}

impl PopulationOrdering for LineageMutation {
    fn ordering(population_subsample: &mut Vec<&ScoredUnit>) {
        population_subsample.sort_by(|a, b| b.get_age().partial_cmp(a.get_age()).unwrap())
    }
}
impl GetPopulationPrompt for LineageMutation {
    fn get_prompt(&self, population_subsample: &[&ScoredUnit]) -> String {
        let len = population_subsample.len();
        let prompt_list = Self::format_prompt_list(population_subsample);

        format!(
            "INSTRUCTION GENOTYPES FOUND IN ASCENDING ORDER OF QUALITY\n{prompt_list}\n{}.",
            len + 1
        )
    }
}

#[cfg(test)]
mod test {
    use crate::{
        breeder::{
            operator::{
                EstimationOfDistributionMutation, FirstOrderPromptGeneration, RankAndIndexMutation,
                ZeroOrderPromptGeneration,
            },
            prompt::{MutationPrompt, ProblemDescription, TaskPrompt, ThinkingStyle},
            unit::{Population, ScoredUnit, UnitData},
            DirectMutator, DistributionEstimationMutator,
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
        (TaskPrompt::new(problem_description.to_string()), embedding)
    }

    async fn obtain_unit_data(openai: &OpenAiDelegate, problem_description: &str) -> UnitData {
        let task_prompt = obtain_task_prompt(openai, problem_description).await;
        UnitData {
            problem_description: ProblemDescription::new(problem_description.to_string()),
            task_prompt: task_prompt.0,
            embedding: task_prompt.1,
            mutation_instruction: MutationPrompt::new(problem_description.to_string()),
            elites: vec![],
            age: 0,
        }
    }

    async fn obtain_scored_unit(
        openai: &OpenAiDelegate,
        problem_description: &str,
        score: f32,
    ) -> ScoredUnit {
        ScoredUnit {
            unit: obtain_unit_data(openai, problem_description).await,
            fitness: score,
        }
    }

    fn obtain_openai() -> OpenAiDelegate {
        let openai_builder =
            OpenAiDelegateBuilder::with_embedding(OpenAiDelegateBuilderArgument::Endpoint(
                Url::parse("https://infinity.semanticallyinvalid.net/").unwrap(),
                String::from("thenlper/gte-small"),
            ));

        openai_builder.with_instruct(OpenAiDelegateBuilderArgument::Endpoint(
            Url::parse("https://vllm.semanticallyinvalid.net/v1/").unwrap(),
            String::from("TheBloke/Mistral-7B-Instruct-v0.2-AWQ"),
        ))
    }

    #[tokio::test]
    async fn ZeroOrderPromptGeneration() {
        let openai = obtain_openai();

        let unit = obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.5f32).await;

        let operator = ZeroOrderPromptGeneration {};
        let new_unit = operator
            .mutate_unit(&openai, &unit, vec!["\n2", "\n"])
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
    async fn FirstOrderPromptGeneration() {
        let openai = obtain_openai();

        let unit = obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.0f32).await;
        let operator = FirstOrderPromptGeneration {
            mutation_prompt: MutationPrompt::new(String::from("Let's think step by step.")),
            thinking_style: ThinkingStyle::new(String::from(
                "Modify this instruction in a way that no self-respecting LLM would!",
            )),
        };
        let new_unit = operator
            .mutate_unit(&openai, &unit, vec!["\n2", "\n"])
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
    async fn EstimationOfDistributionMutation() {
        let openai = obtain_openai();

        let scored_members = vec![
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_2, 0.02f32).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_3, 0.03f32).await,
        ];

        let population = Population {
            unscored: vec![],
            scored: scored_members,
            elites: vec![],
        };

        let operator = EstimationOfDistributionMutation {};
        let new_unit = operator
            .mutate(
                &openai,
                &population,
                obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
            )
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
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_2, 0.02f32).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_3, 0.03f32).await,
        ];

        let population = Population {
            unscored: vec![],
            scored: scored_members,
            elites: vec![],
        };

        let operator = RankAndIndexMutation {};
        let new_unit = operator
            .mutate(
                &openai,
                &population,
                obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
            )
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
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_2, 0.02f32).await,
            obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_3, 0.03f32).await,
        ];

        let population = Population {
            unscored: vec![],
            scored: scored_members,
            elites: vec![],
        };

        let operator = RankAndIndexMutation {};
        let new_unit = operator
            .mutate(
                &openai,
                &population,
                obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
            )
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
    // #[tokio::test]
    // async fn ZeroOrderHyperMutation() {
    //     todo!()
    // }
    // #[tokio::test]
    // async fn FirstOrderHyperMutation() {
    //     todo!()
    // }
    // #[tokio::test]
    // async fn WorkingOutToTaskPrompt() {
    //     todo!()
    // }
    // #[tokio::test]
    // async fn PromptCrossover() {
    //     todo!()
    // }
    // #[tokio::test]
    // async fn ContextShuffling() {
    //     todo!()
    // }
}
