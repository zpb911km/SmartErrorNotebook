use serde::Serialize;

use crate::domain::model::Tag;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TagData {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) color: String,
}

impl From<Tag> for TagData {
    fn from(value: Tag) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            color: value.color,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagResponse {
    pub(crate) tag: TagData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTagResponse {
    pub(crate) tag: TagData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTagResponse {
    pub(crate) id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTagsResponse {
    pub(crate) tags: Vec<TagData>,
}
