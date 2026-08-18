pub struct QuestionQuery {
    pub subject_id: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
pub struct NewQuestion {
    pub id: String,
    pub user_id: String,
    pub subject_id: String,
    pub source_id: Option<String>,
    pub prompt: String,
    pub type_: String,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
    pub now: i64,
}
pub struct QuestionChanges {
    pub id: String,
    pub subject_id: Option<String>,
    pub source_id: Option<String>,
    pub prompt: Option<String>,
    pub type_: Option<String>,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
    pub now: i64,
}
pub struct SyncedQuestion {
    pub id: String,
    pub version: i32,
    pub deleted_at: Option<i64>,
    pub user_id: String,
    pub subject_id: String,
    pub source_id: Option<String>,
    pub prompt: String,
    pub type_: String,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
    pub sync_hash: Option<String>,
    pub now: i64,
}
