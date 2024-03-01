use crate::{
    breeder::{prompt::ThinkingStyle, unit::Unit, ScoredUnit},
    openai::OpenAiDelegate,
};
pub(crate) struct ZeroOrderHyperMutation {
    pub(crate) thinking_style: ThinkingStyle,
}
impl ZeroOrderHyperMutation {
    fn prompt_for_task_prompt(&self, unit: &ScoredUnit, _openai: &OpenAiDelegate) -> String {
        format!(
            "INSTRUCTION: {}\n{}\nINSTRUCTION MUTANT:",
            self.thinking_style,
            unit.get_task_prompt()
        )
    }
}
