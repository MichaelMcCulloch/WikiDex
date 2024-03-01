use std::marker::PhantomData;

#[derive(Clone)]
pub(crate) struct Prompt(pub(crate) String);

impl std::fmt::Display for Prompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Marker types

#[derive(Clone)]
pub(crate) struct TaskMarker;

#[derive(Clone)]
pub(crate) struct MutationMarker;

#[derive(Clone)]
pub(crate) struct HyperMutationMarker;

#[derive(Clone)]
pub(crate) struct ProblemDescriptionMarker;

#[derive(Clone)]
pub(crate) struct ThinkingStyleMarker;

// Generic wrapper
#[derive(Clone)]
pub(crate) struct PromptWrapper<T> {
    pub(crate) prompt: Prompt,
    _marker: std::marker::PhantomData<T>,
}

impl<T> std::fmt::Display for PromptWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.prompt)
    }
}

// Type aliases for readability
pub(crate) type TaskPrompt = PromptWrapper<TaskMarker>;
pub(crate) type MutationPrompt = PromptWrapper<MutationMarker>;
pub(crate) type HyperMutationPrompt = PromptWrapper<HyperMutationMarker>;
pub(crate) type ProblemDescription = PromptWrapper<ProblemDescriptionMarker>;
pub(crate) type ThinkingStyle = PromptWrapper<ThinkingStyleMarker>;

impl Prompt {
    pub(crate) fn new<S: AsRef<str>>(prompt: S) -> Self {
        Self(String::from(prompt.as_ref()))
    }
}
impl<T> PromptWrapper<T> {
    pub(crate) fn new<S: AsRef<str>>(prompt: S) -> Self {
        Self {
            prompt: Prompt::new(prompt),
            _marker: PhantomData,
        }
    }
}
