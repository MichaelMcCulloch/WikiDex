use crate::{breeder::unit::ScoredUnit};

pub(crate) trait PromptForTaskPrompt {
    fn prompt_for_task_prompt(&self, unit: &ScoredUnit) -> String;
}
