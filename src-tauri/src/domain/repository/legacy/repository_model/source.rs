use crate::domain::model::Source;

pub struct SourceWithContext {
    pub source: Source,
    pub question_id: Option<String>,
}

#[derive(Clone)]
pub struct SourceValues {
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}
pub struct NewSource {
    pub id: String,
    pub values: SourceValues,
    pub now: i64,
}
pub struct SourceChanges {
    pub id: String,
    pub values: SourceValues,
    pub now: i64,
}
pub struct SyncedSource {
    pub id: String,
    pub version: i32,
    pub deleted_at: Option<i64>,
    pub question_id: Option<String>,
    pub values: SourceValues,
    pub now: i64,
}
