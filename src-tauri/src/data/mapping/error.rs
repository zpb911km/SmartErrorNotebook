use thiserror::Error;

use crate::domain::model::error::DomainError;

#[derive(Debug, Error)]
pub enum MappingError {
    #[error("failed to convert string to enum: {0}")]
    StringToEnum(&'static str),
    #[error("failed to deserialize feedback history: {0}")]
    DeserializeFeedbackHistory(#[source] serde_json::Error),
    #[error(transparent)]
    Domain(#[from] DomainError),
}
