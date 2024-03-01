mod context;
mod crossover;
mod eda;
mod first_order_meta;
mod first_order_prompt;
mod lineage;
mod rank;
mod work_out;
mod zero_order_meta;
mod zero_order_prompt;

// #[cfg(test)]
// mod test {
//     use crate::{
//         breeder::{
//             prompt::{MutationPrompt, ProblemDescription, TaskPrompt},
//             unit::{Population, ScoredUnit, UnitData},
//         },
//         openai::{OpenAiDelegate, OpenAiDelegateBuilder, OpenAiDelegateBuilderArgument},
//     };
//     use url::Url;

//     const PROBLEM_DESCRIPTION: &str = "Pour water out of a boot.";
//     const PROBLEM_DESCRIPTION_2: &str = "Evacuate the moisture from footwear.";
//     const PROBLEM_DESCRIPTION_3: &str = "Dry the sandals.";

//     async fn obtain_task_prompt(
//         openai: &OpenAiDelegate,
//         problem_description: &str,
//     ) -> (TaskPrompt, Vec<f32>) {
//         let embedding = openai.embed(problem_description).await.unwrap();
//         (TaskPrompt::new(problem_description.to_string()), embedding)
//     }

//     async fn obtain_unit_data(openai: &OpenAiDelegate, problem_description: &str) -> UnitData {
//         let task_prompt = obtain_task_prompt(openai, problem_description).await;
//         UnitData {
//             problem_description: ProblemDescription::new(problem_description.to_string()),
//             task_prompt: task_prompt.0,
//             embedding: task_prompt.1,
//             mutation_instruction: MutationPrompt::new(problem_description.to_string()),
//             elites: vec![],
//             age: 0,
//         }
//     }

//     async fn obtain_scored_unit(
//         openai: &OpenAiDelegate,
//         problem_description: &str,
//         score: f32,
//     ) -> ScoredUnit {
//         ScoredUnit {
//             unit: obtain_unit_data(openai, problem_description).await,
//             fitness: score,
//             elites: vec![],
//         }
//     }

//     fn obtain_openai() -> OpenAiDelegate {
//         let openai_builder =
//             OpenAiDelegateBuilder::with_embedding(OpenAiDelegateBuilderArgument::Endpoint(
//                 Url::parse("https://infinity.semanticallyinvalid.net/").unwrap(),
//                 String::from("thenlper/gte-small"),
//             ));

//         openai_builder.with_instruct(OpenAiDelegateBuilderArgument::Endpoint(
//             Url::parse("https://vllm.semanticallyinvalid.net/v1/").unwrap(),
//             String::from("TheBloke/Mistral-7B-Instruct-v0.2-AWQ"),
//         ))
//     }

//     #[tokio::test]
//     async fn ZeroOrderPromptGeneration() {
//         let openai = obtain_openai();

//         let unit = obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.5f32).await;

//         let operator = ZeroOrderPromptGeneration {};
//         let new_unit = operator
//             .mutate_unit(&openai, &unit, vec!["\n2", "\n"])
//             .await;

//         match new_unit {
//             Ok(mutant) => {
//                 println!("{mutant}");
//             }
//             Err(e) => {
//                 println!("{e}")
//             }
//         };
//     }

//     #[tokio::test]
//     async fn FirstOrderPromptGeneration() {
//         let openai = obtain_openai();

//         let unit = obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.0f32).await;
//         let operator = FirstOrderPromptGeneration {
//             mutation_prompt: MutationPrompt::new(String::from(
//                 "Modify this instruction in a way that no self-respecting LLM would!",
//             )),
//         };
//         let new_unit = operator
//             .mutate_unit(&openai, &unit, vec!["\n2", "\n"])
//             .await;

//         match new_unit {
//             Ok(mutant) => {
//                 println!("{mutant}");
//             }
//             Err(e) => {
//                 println!("{e}")
//             }
//         };
//     }

//     #[tokio::test]
//     async fn EstimationOfDistributionMutation() {
//         let openai = obtain_openai();

//         let scored_members = vec![
//             obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
//             obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_2, 0.02f32).await,
//             obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_3, 0.03f32).await,
//         ];

//         let population = Population {
//             unscored: vec![],
//             scored: scored_members,
//         };

//         let operator = EstimationOfDistributionMutation {};
//         let new_unit = operator
//             .mutate(
//                 &openai,
//                 &population,
//                 obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
//             )
//             .await;

//         match new_unit {
//             Ok(mutant) => {
//                 println!("{mutant}");
//             }
//             Err(e) => {
//                 println!("{e}")
//             }
//         };
//     }
//     #[tokio::test]
//     async fn RankAndIndexMutation() {
//         let openai = obtain_openai();

//         let scored_members = vec![
//             obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
//             obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_2, 0.02f32).await,
//             obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_3, 0.03f32).await,
//         ];

//         let population = Population {
//             unscored: vec![],
//             scored: scored_members,
//         };

//         let operator = RankAndIndexMutation {};
//         let new_unit = operator
//             .mutate(
//                 &openai,
//                 &population,
//                 obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
//             )
//             .await;

//         match new_unit {
//             Ok(mutant) => {
//                 println!("{mutant}");
//             }
//             Err(e) => {
//                 println!("{e}")
//             }
//         };
//     }
//     #[tokio::test]
//     async fn LineageBasedMutation() {
//         let openai = obtain_openai();

//         let scored_members = vec![
//             obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
//             obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_2, 0.02f32).await,
//             obtain_scored_unit(&openai, PROBLEM_DESCRIPTION_3, 0.03f32).await,
//         ];

//         let population = Population {
//             unscored: vec![],
//             scored: scored_members,
//         };

//         let operator = RankAndIndexMutation {};
//         let new_unit = operator
//             .mutate(
//                 &openai,
//                 &population,
//                 obtain_scored_unit(&openai, PROBLEM_DESCRIPTION, 0.01f32).await,
//             )
//             .await;

//         match new_unit {
//             Ok(mutant) => {
//                 println!("{mutant}");
//             }
//             Err(e) => {
//                 println!("{e}")
//             }
//         };
//     }
//     // #[tokio::test]
//     // async fn ZeroOrderHyperMutation() {
//     //     todo!()
//     // }
//     // #[tokio::test]
//     // async fn FirstOrderHyperMutation() {
//     //     todo!()
//     // }
//     // #[tokio::test]
//     // async fn WorkingOutToTaskPrompt() {
//     //     todo!()
//     // }
//     // #[tokio::test]
//     // async fn PromptCrossover() {
//     //     todo!()
//     // }
//     // #[tokio::test]
//     // async fn ContextShuffling() {
//     //     todo!()
//     // }
// }
