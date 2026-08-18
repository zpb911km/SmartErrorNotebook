mod sync;

pub use sync::SyncStatus;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("Unknown sync status: {0}")]
    UnknownSyncStatus(String),
}
