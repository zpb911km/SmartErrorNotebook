use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateSubjectInput {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateSubjectInput {
    pub id: String,
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpsertSubjectInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub name: String,
    pub color: Option<String>,
}
