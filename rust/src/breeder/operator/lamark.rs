use crate::breeder::{
    mutator::{direct::PromptForTaskPrompt, stop_sequences::StopSequences},
    ScoredUnit,
};

pub(crate) struct WorkingOutToTaskPromptMutation {
    pub(crate) correct_solution: String,
}
impl PromptForTaskPrompt for WorkingOutToTaskPromptMutation {
    fn prompt_for_task_prompt(&self, _unit: &ScoredUnit) -> String {
        format!(
            "I gave a friend an instruction and some advice. Here are the correct examples of his workings out:\nCorrect Solution\n{}\n\nThe instruction was:\n",
            self.correct_solution,
        )
    }
}
impl StopSequences for WorkingOutToTaskPromptMutation {
    fn stop_sequence() -> Vec<String> {
        vec![]
    }
}

#[cfg(test)]
mod test {
    use super::WorkingOutToTaskPromptMutation;
    use crate::{
        breeder::{
            mutator::direct::DirectMutator,
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
            mutation_instruction: MutationPrompt::new(problem_description),
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
    async fn WorkingOutToTaskPrompt() {
        let openai = obtain_openai();

        let unit = obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.0f32).await;
        let operator = WorkingOutToTaskPromptMutation {
            correct_solution: String::from("2+2=4"),
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
    // #[tokio::test]
    // async fn PromptCrossover() {
    //     todo!()
    // }
    // #[tokio::test]
    // async fn ContextShuffling() {
    //     todo!()
    // }
}
