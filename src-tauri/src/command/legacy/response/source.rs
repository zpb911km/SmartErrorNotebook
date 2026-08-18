use super::metadata::MetadataOutput;
use crate::model::Source;
use serde::Serialize;

#[derive(Serialize)]
pub struct SourceOutput {
    pub id: String,
    pub question_id: Option<String>,
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
    #[serde(flatten)]
    pub metadata: MetadataOutput,
}
impl SourceOutput {
    pub fn new(value: Source, question_id: Option<String>) -> Self {
        let subject_id = value.subject_id.map(|id| id.to_string());
        Self {
            id: value.id.to_string(),
            question_id,
            subject_id,
            book: value.book,
            chapter: value.chapter,
            knowledge: value.knowledge,
            metadata: (&value.metadata).into(),
        }
    }
}
