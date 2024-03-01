use crate::{breeder::unit::ScoredUnit};

pub(crate) trait PromptForMutatorPrompt {
    fn prompt_for_meta_prompt(&self, unit: &ScoredUnit) -> String;
}
