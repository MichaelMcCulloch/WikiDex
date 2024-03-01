use crate::{breeder::unit::ScoredUnit, openai::OpenAiDelegate};

pub(crate) trait PromptForTaskPrompt {
    fn prompt_for_task_prompt(&self, unit: &ScoredUnit, openai: &OpenAiDelegate) -> String;
}
