use crate::breeder::mutator::{GetPopulationPrompt, PopulationOrdering, PopulationSelector};
use crate::breeder::unit::{Population, ScoredUnit, Unit};
pub(crate) struct LineageMutation {}
impl PopulationSelector for LineageMutation {
    fn select<'a>(_population: &'a Population, unit: &'a ScoredUnit) -> Vec<&'a ScoredUnit> {
        unit.elites.iter().collect::<Vec<_>>()
    }
}

impl PopulationOrdering for LineageMutation {
    fn ordering(population_subsample: &mut Vec<&ScoredUnit>) {
        population_subsample.sort_by(|a, b| a.get_age().partial_cmp(b.get_age()).unwrap())
    }
}
impl GetPopulationPrompt for LineageMutation {
    fn get_prompt(&self, population_subsample: &[&ScoredUnit]) -> String {
        let len = population_subsample.len();
        let prompt_list = Self::format_prompt_list(population_subsample);

        format!(
            "INSTRUCTION GENOTYPES FOUND IN ASCENDING ORDER OF QUALITY\n{prompt_list}\n{}.",
            len + 1
        )
    }
}
