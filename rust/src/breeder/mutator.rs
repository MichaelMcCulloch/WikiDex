use rand::seq::SliceRandom;

use crate::{
    breeder::engine::TaskPrompt,
    openai::{LanguageServiceArguments, LlmMessage, OpenAiDelegate},
};

use super::PromptBreedingError;

pub(crate) enum Operator {
    ZeroOrderPromptGeneration(TaskPrompt),
    FirstOrderPromptGeneration(String, String, TaskPrompt),
    EstimationOfDistributionMutation(Vec<TaskPrompt>),
    RankAndIndexMutation,
    LineageBasedMutation,
    ZeroOrderHyperMutation,
    FirstOrderHyperMutation,
    WorkingOutToTaskPrompt,
    PromptCrossover,
    ContextShuffling,
}

impl Operator {
    pub(crate) async fn ask_llm(
        openai: &OpenAiDelegate,
        mutation_instruction: &str,
        citation_index_begin: usize,
        stop_phrases: Vec<&str>,
    ) -> Result<(String, Vec<f32>), PromptBreedingError> {
        let content = openai
            .get_llm_answer(
                LanguageServiceArguments {
                    system: mutation_instruction,
                    documents: "",
                    query: "",
                    citation_index_begin,
                },
                128u16,
                stop_phrases,
            )
            .await
            .map(|LlmMessage { role: _, content }| content)
            .map_err(PromptBreedingError::LlmError)?;
        let embedding = openai.embed(&content).await.unwrap();
        Ok((content, embedding))
    }

    pub(crate) async fn new_task_prompt(
        self,
        openai: &OpenAiDelegate,
    ) -> Result<Vec<TaskPrompt>, PromptBreedingError> {
        match self {
            Operator::ZeroOrderPromptGeneration(problem_description) => {
                let mutation_instruction =
                    format!("{problem_description} A list of 100 hints:\n1. ");

                let content =
                    Operator::ask_llm(openai, &mutation_instruction, 0, vec!["\n2"]).await?;
                Ok(vec![TaskPrompt {
                    task_prompt: content
                        .0
                        .trim()
                        .trim_start_matches("1. ")
                        .trim()
                        .to_string(),
                    embedding: content.1,
                    fitness_score: None,
                }])
            }
            Operator::FirstOrderPromptGeneration(style, mutation, problem_description) => {
                let mutation_instruction = format!(
                    "MUTATION: {mutation} {style}\nINSTRUCTION: {problem_description}\nINSTRUCTION MUTANT:"
                );

                let content =
                    Operator::ask_llm(openai, &mutation_instruction, 0, vec!["\n"]).await?;
                Ok(vec![TaskPrompt {
                    task_prompt: content.0.trim().to_string(),
                    embedding: content.1,
                    fitness_score: None,
                }])
            }
            Operator::EstimationOfDistributionMutation(population_subset) => {
                let mut new_task_prompts = vec![];
                for _ in 0..5 {
                    let mut aggregate_task_prompts = vec![];
                    aggregate_task_prompts.extend(&population_subset);
                    aggregate_task_prompts.extend(&new_task_prompts);
                    aggregate_task_prompts.shuffle(&mut rand::thread_rng());
                    let len = aggregate_task_prompts.len();
                    let prompt_list = aggregate_task_prompts
                        .into_iter()
                        .enumerate()
                        .map(|(index, task_prompt)| format!("{}. {task_prompt}", index + 1))
                        .collect::<Vec<_>>()
                        .join("\n");

                    let mutation_instruction = format!(
                        "Continue the series with more items:\n{prompt_list}\n{}.",
                        len + 1
                    );

                    let content =
                        Operator::ask_llm(openai, &mutation_instruction, 0, vec!["\n"]).await?;

                    let new_task_prompt = TaskPrompt {
                        task_prompt: content.0.trim().to_string(),
                        embedding: content.1,
                        fitness_score: None,
                    };
                    new_task_prompts.push(new_task_prompt);
                }
                Ok(new_task_prompts)
            }
            Operator::RankAndIndexMutation => todo!(),
            Operator::LineageBasedMutation => todo!(),
            Operator::ZeroOrderHyperMutation => todo!(),
            Operator::FirstOrderHyperMutation => todo!(),
            Operator::WorkingOutToTaskPrompt => todo!(),
            Operator::PromptCrossover => todo!(),
            Operator::ContextShuffling => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Operator;
    use crate::{
        breeder::engine::TaskPrompt,
        openai::{OpenAiDelegate, OpenAiDelegateBuilder, OpenAiDelegateBuilderArgument},
    };
    use url::Url;

    const PROBLEM_DESCRIPTION: &str = "Pour water out of a boot.";
    const PROBLEM_DESCRIPTION_2: &str = "Evacuate the moisture from footwear.";
    const PROBLEM_DESCRIPTION_3: &str = "Dry the sandals.";

    async fn obtain_task_prompt(openai: &OpenAiDelegate, problem_description: &str) -> TaskPrompt {
        let embedding = openai.embed(problem_description).await.unwrap();
        TaskPrompt::new(problem_description, embedding)
    }

    fn obtain_openai() -> OpenAiDelegate {
        let openai_builder =
            OpenAiDelegateBuilder::with_embedding(OpenAiDelegateBuilderArgument::Endpoint(
                Url::parse("http://0.0.0.0:9000/").unwrap(),
                String::from("thenlper/gte-small"),
            ));

        openai_builder.with_instruct(OpenAiDelegateBuilderArgument::Endpoint(
            Url::parse("http://0.0.0.0:5050/v1/").unwrap(),
            String::from("TheBloke/Mistral-7B-Instruct-v0.2-AWQ"),
        ))
    }

    #[tokio::test]
    async fn ZeroOrderPromptGeneration() {
        let openai = obtain_openai();

        let task_prompt = obtain_task_prompt(&openai, PROBLEM_DESCRIPTION).await;

        let operator = Operator::ZeroOrderPromptGeneration(task_prompt);

        match operator.new_task_prompt(&openai).await {
            Ok(mutants) => {
                for mutant in mutants {
                    println!("{mutant}");
                }
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }

    #[tokio::test]
    async fn FirstOrderPromptGeneration() {
        let openai = obtain_openai();

        let task_prompt = obtain_task_prompt(&openai, PROBLEM_DESCRIPTION).await;
        let operator = Operator::FirstOrderPromptGeneration(
            String::from("Let's think step by step."),
            String::from("Modify this instruction in a way that no self-respecting LLM would!"),
            task_prompt,
        );

        match operator.new_task_prompt(&openai).await {
            Ok(mutants) => {
                for mutant in mutants {
                    println!("{mutant}");
                }
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }

    #[tokio::test]
    async fn EstimationOfDistributionMutation() {
        let openai = obtain_openai();

        let operator = Operator::EstimationOfDistributionMutation(vec![
            obtain_task_prompt(&openai, PROBLEM_DESCRIPTION).await,
            obtain_task_prompt(&openai, PROBLEM_DESCRIPTION_2).await,
            obtain_task_prompt(&openai, PROBLEM_DESCRIPTION_3).await,
        ]);

        match operator.new_task_prompt(&openai).await {
            Ok(mutants) => {
                for mutant in mutants {
                    println!("{mutant}");
                }
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }
    #[tokio::test]
    async fn RankAndIndexMutation() {
        let openai: OpenAiDelegate = obtain_openai();

        let operator = Operator::RankAndIndexMutation;

        match operator.new_task_prompt(&openai).await {
            Ok(mutants) => {
                for mutant in mutants {
                    println!("{mutant}");
                }
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }
    #[tokio::test]
    async fn LineageBasedMutation() {
        let openai = obtain_openai();

        let operator = Operator::LineageBasedMutation;

        match operator.new_task_prompt(&openai).await {
            Ok(mutants) => {
                for mutant in mutants {
                    println!("{mutant}");
                }
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }
    #[tokio::test]
    async fn ZeroOrderHyperMutation() {
        let openai: OpenAiDelegate = obtain_openai();

        let operator = Operator::ZeroOrderHyperMutation;

        match operator.new_task_prompt(&openai).await {
            Ok(mutants) => {
                for mutant in mutants {
                    println!("{mutant}");
                }
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }
    #[tokio::test]
    async fn FirstOrderHyperMutation() {
        let openai = obtain_openai();

        let operator = Operator::FirstOrderHyperMutation;

        match operator.new_task_prompt(&openai).await {
            Ok(mutants) => {
                for mutant in mutants {
                    println!("{mutant}");
                }
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }
    #[tokio::test]
    async fn WorkingOutToTaskPrompt() {
        let openai = obtain_openai();

        let operator = Operator::WorkingOutToTaskPrompt;

        match operator.new_task_prompt(&openai).await {
            Ok(mutants) => {
                for mutant in mutants {
                    println!("{mutant}");
                }
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }
    #[tokio::test]
    async fn PromptCrossover() {
        let openai = obtain_openai();

        let operator = Operator::PromptCrossover;

        match operator.new_task_prompt(&openai).await {
            Ok(mutants) => {
                for mutant in mutants {
                    println!("{mutant}");
                }
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }
    #[tokio::test]
    async fn ContextShuffling() {
        let openai = obtain_openai();

        let operator = Operator::ContextShuffling;

        match operator.new_task_prompt(&openai).await {
            Ok(mutants) => {
                for mutant in mutants {
                    println!("{mutant}");
                }
            }
            Err(e) => {
                println!("{e}")
            }
        };
    }
}
