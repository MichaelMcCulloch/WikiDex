pub(crate) mod r#async;
mod error;
mod service;
pub(crate) mod sync;

pub(crate) use error::EmbeddingServiceError;
pub(crate) use service::{EmbedService, EmbedServiceSync};
