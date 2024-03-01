use crate::{
    breeder::{mutator::PromptForTaskPrompt, prompt::MutationPrompt, unit::Unit, ScoredUnit},
    openai::OpenAiDelegate,
};
pub(crate) struct FirstOrderPromptGeneration {
    pub(crate) mutation_prompt: MutationPrompt,
}
pub(crate) struct ZeroOrderPromptGeneration {}
impl PromptForTaskPrompt for ZeroOrderPromptGeneration {
    fn prompt_for_task_prompt(&self, unit: &ScoredUnit, _openai: &OpenAiDelegate) -> String {
        format!(
            "INSTRUCTION: {}\nA list of 100 hints:\n1. ",
            unit.get_problem_description()
        )
    }
}
