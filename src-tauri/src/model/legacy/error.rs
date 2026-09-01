#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Unknown sync status: {status}")]
pub struct UnknownSyncStatus {
    pub(crate) status: String,
}
