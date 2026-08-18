use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSourceInput {
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateSourceInput {
    pub id: String,
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpsertSourceInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub question_id: Option<String>,
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SourceFilter {
    pub subject_id: Option<String>,
}
