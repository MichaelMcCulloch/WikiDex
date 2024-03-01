use crate::breeder::{unit::Unit, ScoredUnit};

pub(crate) trait GetPopulationPrompt {
    fn get_prompt(&self, population_subsample: &[&ScoredUnit]) -> String;
    fn format_prompt_list(population_subsample: &[&ScoredUnit]) -> String {
        population_subsample
            .iter()
            .enumerate()
            .map(|(index, unit)| format!("{}. {}", index + 1, unit.get_task_prompt()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
