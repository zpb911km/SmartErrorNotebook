use serde::Serialize;

use crate::application::UseCaseError;
use crate::domain::repository::error::{
    RepositoryDeleteError, RepositoryError, RepositoryFindError, RepositorySaveError,
};
use crate::util::parsing::ValueParseError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl ApiError {
    pub(super) fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: "INVALID_ARGUMENT",
            message: message.into(),
            details: None,
        }
    }

    pub(super) fn not_found(entity: &'static str, id: Option<uuid::Uuid>) -> Self {
        Self {
            code: "NOT_FOUND",
            message: format!("{entity} was not found"),
            details: Some(match id {
                Some(id) => serde_json::json!({ "entity": entity, "id": id }),
                None => serde_json::json!({ "entity": entity }),
            }),
        }
    }

    fn repository_error(error: RepositoryError) -> Self {
        match error {
            RepositoryError::Save(error) => Self::save_error(error),
            RepositoryError::Delete(error) => Self::delete_error(error),
            RepositoryError::Find(error) => Self::find_error(error),
            RepositoryError::Count(_) => Self::storage_error(),
        }
    }

    fn save_error(error: RepositorySaveError) -> Self {
        match error {
            RepositorySaveError::MissingReference(error) => Self {
                code: "MISSING_REFERENCE",
                message: error.to_string(),
                details: Some(serde_json::json!({
                    "entity": error.owner.entity,
                    "id": error.owner.id,
                    "missing": error.missing.iter().map(|item| serde_json::json!({"entity": item.entity, "id": item.id})).collect::<Vec<_>>()
                })),
            },
            RepositorySaveError::Referenced(error) => Self {
                code: "RESOURCE_IN_USE",
                message: error.to_string(),
                details: Some(
                    serde_json::json!({"entity": error.target.entity, "id": error.target.id, "referencedBy": error.referenced_by}),
                ),
            },
            RepositorySaveError::Infrastructure(_) => Self::storage_error(),
        }
    }

    fn delete_error(error: RepositoryDeleteError) -> Self {
        match error {
            RepositoryDeleteError::Referenced(error) => Self {
                code: "RESOURCE_IN_USE",
                message: error.to_string(),
                details: Some(
                    serde_json::json!({"entity": error.target.entity, "id": error.target.id, "referencedBy": error.referenced_by}),
                ),
            },
            RepositoryDeleteError::Infrastructure(_) => Self::storage_error(),
        }
    }

    fn find_error(error: RepositoryFindError) -> Self {
        match error {
            RepositoryFindError::CorruptedData(error) => Self {
                code: "CORRUPTED_DATA",
                message: error.to_string(),
                details: None,
            },
            RepositoryFindError::Infrastructure(_) => Self::storage_error(),
        }
    }

    fn storage_error() -> Self {
        Self {
            code: "STORAGE_ERROR",
            message: "storage operation failed".into(),
            details: None,
        }
    }
}

impl From<ValueParseError> for ApiError {
    fn from(value: ValueParseError) -> Self {
        Self::invalid(value.to_string())
    }
}

impl From<UseCaseError> for ApiError {
    fn from(value: UseCaseError) -> Self {
        match value {
            UseCaseError::NotFound { entity, id } => Self::not_found(entity, id),
            UseCaseError::Domain(error) => Self::invalid(error.to_string()),
            UseCaseError::Repository(error) => Self::repository_error(error),
        }
    }
}
