use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TagInfo {
    pub name: String,
    pub color: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateErrorTagsForQuestionInput {
    pub question_id: String,
    pub tags: Vec<TagInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct UpsertErrorTagInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    #[serde(default)]
    pub question_ids: Vec<String>,
    #[serde(default)]
    pub question_id: Option<String>,
    pub name: String,
    pub color: String,
}
