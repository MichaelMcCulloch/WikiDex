use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::unbounded_channel;

use crate::{
    docstore::SqliteDocstore,
    formatter::CitationStyle,
    index::FaissIndex,
    openai::{LanguageServiceArguments, LlmMessage, OpenAiDelegate},
};

use super::PromptBreedingError;

pub struct Engine {
    index: Mutex<FaissIndex>,
    openai: Arc<OpenAiDelegate>,
    docstore: SqliteDocstore,
    // thinking_styles: SqliteDocstore,
    // mutation_prompts: SqliteDocstore,
}

pub struct Unit {
    // task_prompts: Vec<String>,
    mutation_prompt: String,
    fitness_score: Option<f32>, // Fitness could be an option if not yet evaluated
}

const NUM_DOCUMENTS_TO_RETRIEVE: usize = 4;

const CITATION_STYLE: CitationStyle = CitationStyle::MLA;

impl Engine {
    pub(crate) async fn query(
        &self,
        question: &str,
        system_prompt: &str,
    ) -> Result<String, PromptBreedingError> {
        let formatted_documents = self.get_documents(question, 0usize).await?;

        let llm_service_arguments = LanguageServiceArguments {
            system: system_prompt,
            documents: &formatted_documents,
            query: question,
            citation_index_begin: 0,
        };

        let LlmMessage { role: _, content } = self
            .openai
            .get_llm_answer(llm_service_arguments, 256u16)
            .await
            .map_err(PromptBreedingError::LlmError)?;

        Ok(content.trim().to_string())
    }

    pub(crate) fn new(
        index: Mutex<FaissIndex>,
        openai: OpenAiDelegate,
        docstore: SqliteDocstore,
        // thinking_styles: SqliteDocstore,
        // mutation_prompts: SqliteDocstore,
    ) -> Self {
        Self {
            index,
            openai: Arc::new(openai),
            docstore,
            // thinking_styles,
            // mutation_prompts,
        }
    }

    pub(crate) async fn get_documents(
        &self,
        _user_query: &str,
        _num_sources_already_in_chat: usize,
    ) -> Result<String, PromptBreedingError> {
        // let embedding = self
        //     .openai
        //     .embed(user_query)
        //     .await
        //     .map_err(PromptBreedingError::EmbeddingServiceError)?;

        // let document_indices = self
        //     .index
        //     .lock()
        //     .map_err(|_| PromptBreedingError::UnableToLockIndex)?
        //     .search(&embedding, NUM_DOCUMENTS_TO_RETRIEVE)
        //     .map_err(PromptBreedingError::IndexError)?;

        // let documents: Vec<(usize, String, crate::formatter::Provenance)> = self
        //     .docstore
        //     .retreive(&document_indices)
        //     .await
        //     .map_err(PromptBreedingError::DocstoreError)?;

        // let formatted_documents = documents
        //     .iter()
        //     .map(|(ordianal, document, provenance)| {
        //         DocumentFormatter::format_document(
        //             *ordianal + num_sources_already_in_chat,
        //             &provenance.title(),
        //             document,
        //         )
        //     })
        //     .collect::<Vec<String>>()
        //     .join("\n\n");

        // Ok(formatted_documents)
        todo!()
    }

    async fn initialize_population(
        &self,
        thinking_styles: &[String],
        mutation_prompts: &[String],
        problem_description: &'static str,
    ) -> Result<Vec<String>, PromptBreedingError> {
        let (tx, mut rx) = unbounded_channel();
        let population_writer = actix_web::rt::spawn(async move {
            let mut intial_population = vec![];
            while let Some(result) = rx.recv().await {
                intial_population.push(result);
            }

            Ok::<Vec<String>, PromptBreedingError>(intial_population)
        });
        for style in thinking_styles.iter() {
            for mutation in mutation_prompts.iter() {
                let tx = tx.clone();
                let openai = self.openai.clone();
                let style = style.clone();
                let mutation = mutation.clone();
                actix_web::rt::spawn(async move {
                    let mutation_instruction =
                        format!("{mutation} INSTRUCTION: {style} {problem_description} INSTRUCTION MUTANT: ");

                    match openai
                        .get_llm_answer(
                            LanguageServiceArguments {
                                system: &mutation_instruction,
                                documents: "",
                                query: "",
                                citation_index_begin: 0,
                            },
                            128u16,
                        )
                        .await
                        .map_err(PromptBreedingError::LlmError)
                    {
                        Ok(LlmMessage { role: _, content }) => {
                            tx.clone().send(content).unwrap();
                        }
                        Err(e) => {
                            log::error!("{e}")
                        }
                    };

                    Ok::<(), PromptBreedingError>(())
                });
            }
        }
        drop(tx);
        let intial_population = population_writer.await.unwrap()?;

        Ok(intial_population)
    }

    async fn evaluate_fitness(
        &self,
        _unit: &Unit,
        _problem_description: &str,
    ) -> Result<f32, PromptBreedingError> {
        // Evaluate the fitness of a unit by testing its task prompts against training data
        todo!()
    }

    async fn update_unit_fitness(&self, _unit: &Unit, _fitness: f32) {
        // Update the fitness score of a unit based on the evaluation
        todo!()
    }

    fn select_random_competitor<'a>(&self, _population: &'a [Unit]) -> &'a Unit {
        // Select a random unit from the population to compete with another unit
        todo!()
    }

    async fn mutate_unit(&self, _unit: &Unit) -> Result<Unit, PromptBreedingError> {
        // Mutate a unit using some mutation strategy
        todo!()
    }

    fn replace_unit(&mut self, _unit_to_replace: &Unit, _new_unit: Unit) {
        // Replace a unit in the population with a new mutated unit
        todo!()
    }

    fn find_best_unit(&self, _population: &[Unit]) -> Result<&Unit, PromptBreedingError> {
        // Find the unit with the best fitness in the population
        todo!()
    }

    pub(crate) async fn breed_prompt(
        &self,
        problem_description: &'static str,
        _number_of_generations: usize,
    ) -> Result<String, PromptBreedingError> {
        let population = self
            .initialize_population(
                &[
                    String::from("Make up a systematic answer that makes you look quite clever."),
                    String::from("Draw a diagram representing the connections between documents."),
                    String::from("Let's think step by step."),
                ],
                &[
                    String::from("Change this instruction to make it more fun."),
                    String::from("Mutate the prompt with an unexpected twist."),
                    String::from("Modify the instruction like no self-respecting LLM would."),
                ],
                problem_description,
            )
            .await?;
        for member in population {
            println!("{member}");
        }
        // while number_of_generations > 0 {
        //     for unit in &population {
        //         let fitness = self.evaluate_fitness(unit, problem_description).await?;
        //         self.update_unit_fitness(unit, fitness).await
        //     }

        //     for unit in &mut population {
        //         let competitor_unit = self.select_random_competitor(&population);
        //         if self.fitness(unit)? > self.fitness(&competitor_unit)? {
        //             let new_unit = self.mutate_unit(unit).await?;
        //             self.replace_unit(competitor_unit, new_unit);
        //         }
        //     }

        //     number_of_generations -= 1;
        // }
        Ok(String::from(problem_description))
    }
}
