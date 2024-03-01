use crate::{
    breeder::{
        prompt::{MutationPrompt, ThinkingStyle},
        unit::Unit,
        ScoredUnit,
    },
    openai::OpenAiDelegate,
};

pub(crate) struct FirstOrderHyperMutation {
    pub(crate) mutation_prompt: MutationPrompt,
}
impl FirstOrderHyperMutation {
    fn prompt_for_task_prompt(&self, unit: &ScoredUnit, _openai: &OpenAiDelegate) -> String {
        format!(
            "{}\nINSTRUCTION: {}\nINSTRUCTION MUTANT:",
            self.mutation_prompt,
            unit.get_task_prompt()
        )
    }
}
pub(crate) struct ZeroOrderHyperMutation {
    pub(crate) thinking_style: ThinkingStyle,
}
impl ZeroOrderHyperMutation {
    fn prompt_for_task_prompt(&self, unit: &ScoredUnit, _openai: &OpenAiDelegate) -> String {
        format!(
            "INSTRUCTION: {}\n{}\nINSTRUCTION MUTANT:",
            self.thinking_style,
            unit.get_task_prompt()
        )
    }
}

#[cfg(test)]
mod test {

    use crate::{
        breeder::{
            prompt::{MutationPrompt, ProblemDescription, TaskPrompt},
            unit::{ScoredUnit, UnitData},
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
            elites: vec![],
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
    async fn ZeroOrderHyperMutation() {
        todo!()
    }
    #[tokio::test]
    async fn FirstOrderHyperMutation() {
        todo!()
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
