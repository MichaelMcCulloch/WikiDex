use crate::breeder::{unit::Population, ScoredUnit};

pub(crate) trait PopulationSelector {
    fn select<'a>(population: &'a Population, unit: &'a ScoredUnit) -> Vec<&'a ScoredUnit>;
}
