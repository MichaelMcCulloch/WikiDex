use crate::breeder::{
    mutator::{direct::PromptForTaskPrompt, stop_sequences::StopSequences},
    prompt::MutationPrompt,
    unit::Unit,
    ScoredUnit,
};

pub(crate) struct FirstOrderPromptGeneration {
    pub(crate) mutation_prompt: MutationPrompt,
}
pub(crate) struct ZeroOrderPromptGeneration {}

impl PromptForTaskPrompt for ZeroOrderPromptGeneration {
    fn prompt_for_task_prompt(&self, unit: &ScoredUnit) -> String {
        format!(
            "INSTRUCTION: {}\nA list of 100 hints:\n1. ",
            unit.get_problem_description()
        )
    }
}

impl PromptForTaskPrompt for FirstOrderPromptGeneration {
    fn prompt_for_task_prompt(&self, unit: &ScoredUnit) -> String {
        format!(
            "MUTATION: {}\nINSTRUCTION: {}\nINSTRUCTION MUTANT:",
            self.mutation_prompt,
            unit.get_task_prompt()
        )
    }
}
impl StopSequences for ZeroOrderPromptGeneration {
    fn stop_sequence() -> Vec<String> {
        vec![String::from("\n2")]
    }
}
impl StopSequences for FirstOrderPromptGeneration {
    fn stop_sequence() -> Vec<String> {
        vec![String::from("\n2")]
    }
}
#[cfg(test)]
mod test {

    use crate::{
        breeder::{
            mutator::direct::DirectMutator,
            operator::prompt::{FirstOrderPromptGeneration, ZeroOrderPromptGeneration},
            prompt::{MutationPrompt, ProblemDescription, TaskPrompt},
            unit::{ScoredUnit, UnitData},
        },
        openai::{OpenAiDelegate, OpenAiDelegateBuilder, OpenAiDelegateBuilderArgument},
    };
    use url::Url;

    const PROBLEM_DESCRIPTION: &str = "Pour water out of a boot.";

    async fn obtain_task_prompt(
        openai: &OpenAiDelegate,
        problem_description: &str,
    ) -> (TaskPrompt, Vec<f32>) {
        let embedding = openai.embed(problem_description).await.unwrap();
        (TaskPrompt::new(problem_description), embedding)
    }

    async fn obtain_unit_data(openai: &OpenAiDelegate, problem_description: &str) -> UnitData {
        let task_prompt = obtain_task_prompt(openai, problem_description).await;
        UnitData {
            problem_description: ProblemDescription::new(problem_description),
            task_prompt: task_prompt.0,
            embedding: task_prompt.1,
            mutation_prompt: MutationPrompt::new(problem_description),
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
        let new_unit = operator.mutate(&openai, &unit).await;

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
            mutation_prompt: MutationPrompt::new(
                "Modify this instruction in a way that no self-respecting LLM would!",
            ),
        };
        let new_unit = operator.mutate(&openai, &unit).await;

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
