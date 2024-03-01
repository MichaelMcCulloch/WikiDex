use std::fmt::Display;

use super::prompt::{MutationPrompt, ProblemDescription, TaskPrompt};

#[derive(Clone)]
pub(crate) struct UnitData {
    pub(crate) problem_description: ProblemDescription,
    pub(crate) task_prompt: TaskPrompt,
    pub(crate) embedding: Vec<f32>,
    pub(crate) mutation_instruction: MutationPrompt,
    pub(crate) elites: Vec<TaskPrompt>,
    pub(crate) age: usize,
}

#[derive(Clone)]
pub(crate) struct ScoredUnit {
    pub(crate) unit: UnitData,
    pub(crate) fitness: f32,
}
#[derive(Clone)]
pub(crate) struct UnscoredUnit {
    pub(crate) unit: UnitData,
}
impl Display for UnitData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.task_prompt)
    }
}
impl Display for UnscoredUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.unit)
    }
}
impl Display for ScoredUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.unit)
    }
}

#[derive(Clone)]
pub(crate) struct Population {
    pub(crate) unscored: Vec<UnscoredUnit>,
    pub(crate) scored: Vec<ScoredUnit>,
    pub(crate) elites: Vec<ScoredUnit>,
}
pub trait Unit {
    fn get_problem_description(&self) -> &ProblemDescription;
    fn get_task_prompt(&self) -> &TaskPrompt;
    fn get_embedding(&self) -> &Vec<f32>;
    fn get_mutation_instruction(&self) -> &MutationPrompt;
    fn get_elites(&self) -> &Vec<TaskPrompt>;
    fn get_age(&self) -> &usize;
}

macro_rules! impl_unit_for_containing_unitdata {
    ($($t:ty),+) => {
        $(impl Unit for $t {
            fn get_problem_description(&self) -> &ProblemDescription {
                &self.unit.problem_description
            }

            fn get_task_prompt(&self) -> &TaskPrompt {
                &self.unit.task_prompt
            }

            fn get_embedding(&self) -> &Vec<f32> {
                &self.unit.embedding
            }

            fn get_mutation_instruction(&self) -> &MutationPrompt {
                &self.unit.mutation_instruction
            }

            fn get_elites(&self) -> &Vec<TaskPrompt> {
                &self.unit.elites
            }
            fn get_age(&self) -> &usize {
                &self.unit.age
            }
        })*
    };
}

// Use the macro to implement Unit for both ScoredUnit and UnscoredUnit
impl_unit_for_containing_unitdata!(ScoredUnit, UnscoredUnit);
pub trait Fitness {
    fn get_fitness(&self) -> &f32;
}
impl Fitness for ScoredUnit {
    fn get_fitness(&self) -> &f32 {
        &self.fitness
    }
}
