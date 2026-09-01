use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub(crate) enum RepositoryError {
    #[error(transparent)]
    Save(#[from] RepositorySaveError),
    #[error(transparent)]
    Delete(#[from] RepositoryDeleteError),
    #[error(transparent)]
    Find(#[from] RepositoryFindError),
    #[error(transparent)]
    Count(#[from] RepositoryCountError),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub(crate) enum RepositorySaveError {
    #[error(transparent)]
    MissingReference(#[from] MissingReference),
    #[error(transparent)]
    Referenced(#[from] Referenced),
    #[error(transparent)]
    Infrastructure(#[from] RepositoryInfrastructureError),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub(crate) enum RepositoryDeleteError {
    #[error(transparent)]
    Referenced(#[from] Referenced),
    #[error(transparent)]
    Infrastructure(#[from] RepositoryInfrastructureError),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub(crate) enum RepositoryFindError {
    #[error(transparent)]
    CorruptedData(#[from] CorruptedData),
    #[error(transparent)]
    Infrastructure(#[from] RepositoryInfrastructureError),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub(crate) enum RepositoryCountError {
    #[error(transparent)]
    Infrastructure(#[from] RepositoryInfrastructureError),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{owner:?} refers to missing models: {missing:?}")]
pub(crate) struct MissingReference {
    pub owner: EntityReference,
    pub missing: Vec<EntityReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{target:?} cannot be deleted because it is referenced by {referenced_by}")]
pub(crate) struct Referenced {
    pub target: EntityReference,
    pub referenced_by: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EntityReference {
    pub entity: &'static str,
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("persisted {entity} `{id}` is corrupted: {message}")]
pub(crate) struct CorruptedData {
    pub(crate) entity: &'static str,
    pub(crate) id: Uuid,
    pub(crate) message: String,
}
impl CorruptedData {
    pub(crate) fn new(entity: &'static str, id: Uuid, error: impl std::error::Error) -> Self {
        Self {
            entity,
            id,
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("repository operation `{operation}` failed: {message}")]
pub(crate) struct RepositoryInfrastructureError {
    pub(crate) operation: &'static str,
    pub(crate) message: String,
}
impl RepositoryInfrastructureError {
    pub(crate) fn new(operation: &'static str, error: impl std::error::Error) -> Self {
        Self {
            operation,
            message: error.to_string(),
        }
    }
}
