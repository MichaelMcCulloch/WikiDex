use crate::{
    breeder::{mutator::PromptForTaskPrompt, prompt::MutationPrompt, unit::Unit, ScoredUnit},
    openai::OpenAiDelegate,
};
pub(crate) struct FirstOrderPromptGeneration {
    pub(crate) mutation_prompt: MutationPrompt,
}
impl PromptForTaskPrompt for FirstOrderPromptGeneration {
    fn prompt_for_task_prompt(&self, unit: &ScoredUnit, _openai: &OpenAiDelegate) -> String {
        format!(
            "MUTATION: {}\nINSTRUCTION: {}\nINSTRUCTION MUTANT:",
            self.mutation_prompt,
            unit.get_task_prompt()
        )
    }
}
