mod builder;
mod chat;
mod completion;
mod delegate;
mod embedding;
mod error;
mod kind;
mod protocol;
mod service;

pub(crate) use builder::{OpenAiDelegateBuilder, OpenAiDelegateBuilderArgument};
pub(crate) use delegate::{LanguageServiceServiceArguments, OpenAiDelegate};
pub(crate) use error::{EmbeddingServiceError, LlmServiceError};
pub(crate) use kind::ModelKind;
pub(crate) use protocol::{LlmMessage, LlmRole, PartialLlmMessage};
pub(crate) use service::{EmbedService, LlmService};
