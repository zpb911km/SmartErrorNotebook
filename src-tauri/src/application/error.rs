use crate::domain::model::error::DomainError;
use crate::domain::repository::error::{
    RepositoryCountError, RepositoryDeleteError, RepositoryError, RepositoryFindError,
    RepositorySaveError,
};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub(crate) enum UseCaseError {
    #[error("{0} was not found")]
    NotFound(&'static str),
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

impl From<RepositorySaveError> for UseCaseError {
    fn from(value: RepositorySaveError) -> Self {
        RepositoryError::from(value).into()
    }
}

impl From<RepositoryDeleteError> for UseCaseError {
    fn from(value: RepositoryDeleteError) -> Self {
        RepositoryError::from(value).into()
    }
}

impl From<RepositoryFindError> for UseCaseError {
    fn from(value: RepositoryFindError) -> Self {
        RepositoryError::from(value).into()
    }
}

impl From<RepositoryCountError> for UseCaseError {
    fn from(value: RepositoryCountError) -> Self {
        RepositoryError::from(value).into()
    }
}
