use serde::Serialize;

use crate::domain::model::Source;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceData {
    pub(crate) id: String,
    pub(crate) subject_id: Option<String>,
    pub(crate) book: Option<String>,
    pub(crate) chapter: Option<String>,
    pub(crate) knowledge: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

impl From<Source> for SourceData {
    fn from(value: Source) -> Self {
        Self {
            id: value.id.to_string(),
            subject_id: value.subject_id.map(|id| id.to_string()),
            book: value.book,
            chapter: value.chapter,
            knowledge: value.knowledge,
            created_at: value.metadata.created_at.to_rfc3339(),
            updated_at: value.metadata.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSourceResponse {
    pub(crate) source: SourceData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSourceResponse {
    pub(crate) source: SourceData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSourceResponse {
    pub(crate) id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSourcesResponse {
    pub(crate) ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSourceResponse {
    pub(crate) source: SourceData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSourcesResponse {
    pub(crate) sources: Vec<SourceData>,
}
