use super::metadata::MetadataOutput;
use crate::model::Tag;
use serde::Serialize;

#[derive(Serialize)]
pub struct TagOutput {
    pub id: String,
    pub question_id: String,
    pub name: String,
    pub color: String,
    #[serde(flatten)]
    pub metadata: MetadataOutput,
}
impl TagOutput {
    pub fn new(value: Tag, question_id: impl Into<String>) -> Self {
        Self {
            id: value.id.to_string(),
            question_id: question_id.into(),
            name: value.name,
            color: value.color,
            metadata: (&value.metadata).into(),
        }
    }
}
