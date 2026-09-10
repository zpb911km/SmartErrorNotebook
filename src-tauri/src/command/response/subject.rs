use serde::Serialize;

use crate::domain::model::Subject;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubjectData {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) color: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

impl From<Subject> for SubjectData {
    fn from(value: Subject) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            color: value.color,
            created_at: value.metadata.created_at.to_rfc3339(),
            updated_at: value.metadata.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubjectResponse {
    pub(crate) subject: SubjectData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubjectResponse {
    pub(crate) subject: SubjectData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSubjectResponse {
    pub(crate) id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSubjectsResponse {
    pub(crate) subjects: Vec<SubjectData>,
}
