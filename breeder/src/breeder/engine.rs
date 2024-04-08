use super::PromptBreedingError;
use crate::{
    docstore::SqliteDocstore, formatter::CitationStyle, index::FaceIndex, openai::OpenAiDelegate,
};

use std::{fmt::Display, sync::Arc};

pub(crate) struct Engine {
    index: FaceIndex,
    openai: Arc<OpenAiDelegate>,
    docstore: SqliteDocstore,
    thinking_styles: Vec<String>,
    mutation_prompts: Vec<String>,
}

impl Display for TaskPrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.task_prompt)
    }
}

pub(crate) struct TaskPrompt {
    pub(crate) task_prompt: String,
    pub(crate) embedding: Vec<f32>,
    pub(crate) fitness_score: Option<f32>, // Fitness could be an option if not yet evaluated
}

impl TaskPrompt {
    pub(crate) fn new(problem_description: &str, embedding: Vec<f32>) -> TaskPrompt {
        TaskPrompt {
            task_prompt: String::from(problem_description),
            embedding,
            fitness_score: None,
        }
    }
}

const NUM_DOCUMENTS_TO_RETRIEVE: usize = 4;

const CITATION_STYLE: CitationStyle = CitationStyle::MLA;

impl Engine {
    pub(crate) fn new(
        index: FaceIndex,
        openai: OpenAiDelegate,
        docstore: SqliteDocstore,
        thinking_styles: Vec<String>,
        mutation_prompts: Vec<String>,
    ) -> Self {
        Self {
            index,
            openai: Arc::new(openai),
            docstore,
            thinking_styles,
            mutation_prompts,
        }
    }

    pub(crate) async fn get_documents(
        &self,
        _user_query: &str,
        _num_sources_already_in_chat: usize,
    ) -> Result<String, PromptBreedingError> {
        todo!()
    }

    async fn initialize_population(
        &self,
        _population_size: usize,
        _thinking_styles: &[String],
        _mutation_prompts: &[String],
        _problem_description: &'static str,
    ) -> Result<Vec<TaskPrompt>, PromptBreedingError> {
        Ok(vec![])
    }

    pub(crate) async fn breed_prompt(
        &self,
        problem_description: &'static str,
        _number_of_generations: usize,
    ) -> Result<String, PromptBreedingError> {
        let _population = self
            .initialize_population(
                50usize,
                &self.thinking_styles,
                &self.mutation_prompts,
                problem_description,
            )
            .await?;

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
