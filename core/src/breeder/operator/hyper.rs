use crate::breeder::{
    mutator::hyper::PromptForMutatorPrompt,
    prompt::{MutationPrompt, ThinkingStyle},
    unit::Unit,
    ScoredUnit,
};

pub(crate) struct FirstOrderHyperMutation {
    pub(crate) mutation_prompt: MutationPrompt,
}
impl PromptForMutatorPrompt for FirstOrderHyperMutation {
    fn prompt_for_meta_prompt(&self, _unit: &ScoredUnit) -> String {
        format!(
            "Please summarize and improve the following instruction: {}",
            self.mutation_prompt
        )
    }
}

pub(crate) struct ZeroOrderHyperMutation {
    pub(crate) thinking_style: ThinkingStyle,
}
impl PromptForMutatorPrompt for ZeroOrderHyperMutation {
    fn prompt_for_meta_prompt(&self, unit: &ScoredUnit) -> String {
        format!("{} {}", unit.get_problem_description(), self.thinking_style)
    }
}

#[cfg(test)]
mod test {
    use super::{FirstOrderHyperMutation, ZeroOrderHyperMutation};
    use crate::{
        breeder::{
            mutator::hyper::MetaMutator,
            prompt::{MutationPrompt, ProblemDescription, TaskPrompt, ThinkingStyle},
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
    async fn ZeroOrderHyperMutation() {
        let openai = obtain_openai();

        let unit = obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.0f32).await;
        let operator = ZeroOrderHyperMutation {
            thinking_style: ThinkingStyle::new("Let's think step by step."),
        };
        let new_unit = operator.mutate(&openai, &unit, vec!["\n2", "\n"]).await;

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
    async fn FirstOrderHyperMutation() {
        let openai = obtain_openai();

        let unit = obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.0f32).await;
        let operator = FirstOrderHyperMutation {
            mutation_prompt: MutationPrompt::new("Modify the following instruction creatively, giving some advice on how to solve it:"),
        };
        let new_unit = operator.mutate(&openai, &unit, vec!["\n2", "\n"]).await;

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
