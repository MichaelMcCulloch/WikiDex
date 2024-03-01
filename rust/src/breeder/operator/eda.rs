use rand::seq::SliceRandom;

use crate::breeder::mutator::{GetPopulationPrompt, PopulationOrdering, PopulationSelector};
use crate::breeder::unit::{Population, ScoredUnit};
pub(crate) struct EstimationOfDistributionMutation {}
impl PopulationSelector for EstimationOfDistributionMutation {
    fn select<'a>(population: &'a Population, _unit: &'a ScoredUnit) -> Vec<&'a ScoredUnit> {
        population.scored.iter().collect::<Vec<_>>()
    }
}
impl PopulationOrdering for EstimationOfDistributionMutation {
    fn ordering(population_subsample: &mut Vec<&ScoredUnit>) {
        population_subsample.shuffle(&mut rand::thread_rng())
    }
}
impl GetPopulationPrompt for EstimationOfDistributionMutation {
    fn get_prompt(&self, population_subsample: &[&ScoredUnit]) -> String {
        let len = population_subsample.len();
        let prompt_list = Self::format_prompt_list(population_subsample);
        format!(
            "A List of responses in random order of score.\n{prompt_list}\n{}.",
            len + 1
        )
    }
}
