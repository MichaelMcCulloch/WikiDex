use crate::breeder::mutator::{GetPopulationPrompt, PopulationOrdering, PopulationSelector};
use crate::breeder::unit::{Population, ScoredUnit};
pub(crate) struct RankAndIndexMutation {}
impl PopulationSelector for RankAndIndexMutation {
    fn select<'a>(population: &'a Population, _unit: &'a ScoredUnit) -> Vec<&'a ScoredUnit> {
        population.scored.iter().collect::<Vec<_>>()
    }
}

impl PopulationOrdering for RankAndIndexMutation {
    fn ordering(population_subsample: &mut Vec<&ScoredUnit>) {
        population_subsample.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap())
    }
}
impl GetPopulationPrompt for RankAndIndexMutation {
    fn get_prompt(&self, population_subsample: &[&ScoredUnit]) -> String {
        let len = population_subsample.len();
        let prompt_list = Self::format_prompt_list(population_subsample);
        format!(
            "A List of responses in descending order of score.\n{prompt_list}\n{}.",
            len + 1
        )
    }
}
