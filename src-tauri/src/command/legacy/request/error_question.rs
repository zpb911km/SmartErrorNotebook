use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateQuestionInput {
    pub user_id: String,
    pub subject_id: String,
    pub source_id: Option<String>,
    pub prompt: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateQuestionInput {
    pub id: String,
    pub subject_id: Option<String>,
    #[serde(alias = "sourceid")]
    pub source_id: Option<String>,
    pub prompt: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpsertQuestionInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    #[serde(alias = "userid")]
    pub user_id: String,
    #[serde(alias = "subjectid")]
    pub subject_id: String,
    #[serde(alias = "sourceid")]
    pub source_id: Option<String>,
    pub prompt: String,
    #[serde(rename = "type", alias = "type_")]
    pub type_: String,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
    pub sync_hash: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct QuestionFilter {
    pub subject_id: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}
