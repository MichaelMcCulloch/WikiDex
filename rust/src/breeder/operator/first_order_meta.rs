use crate::{
    breeder::{prompt::MutationPrompt, unit::Unit, ScoredUnit},
    openai::OpenAiDelegate,
};

pub(crate) struct FirstOrderHyperMutation {
    pub(crate) mutation_prompt: MutationPrompt,
}
impl FirstOrderHyperMutation {
    fn prompt_for_task_prompt(&self, unit: &ScoredUnit, _openai: &OpenAiDelegate) -> String {
        format!(
            "{}\nINSTRUCTION: {}\nINSTRUCTION MUTANT:",
            self.mutation_prompt,
            unit.get_task_prompt()
        )
    }
}
