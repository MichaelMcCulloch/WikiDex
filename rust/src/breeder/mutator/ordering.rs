use crate::breeder::ScoredUnit;

pub(crate) trait PopulationOrdering {
    fn ordering(population_subsample: &mut Vec<&ScoredUnit>);
}
