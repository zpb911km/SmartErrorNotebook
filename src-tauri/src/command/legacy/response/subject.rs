use super::metadata::MetadataOutput;
use crate::model::Subject;
use serde::Serialize;

#[derive(Serialize)]
pub struct SubjectOutput {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    #[serde(flatten)]
    pub metadata: MetadataOutput,
}

impl From<Subject> for SubjectOutput {
    fn from(value: Subject) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            color: (!value.color.is_empty()).then_some(value.color),
            metadata: (&value.metadata).into(),
        }
    }
}
