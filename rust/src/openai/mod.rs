mod builder;
mod chat;
mod delegate;
mod embedding;
mod error;
mod instruct;
mod kind;
mod protocol;

pub(crate) use builder::{OpenAiDelegateBuilder, OpenAiDelegateBuilderArgument};
pub(crate) use delegate::{LanguageServiceArguments, OpenAiDelegate};
pub(crate) use error::{EmbeddingServiceError, LlmServiceError};
pub(crate) use kind::ModelKind;
pub(crate) use protocol::{LlmMessage, LlmRole, PartialLlmMessage};
