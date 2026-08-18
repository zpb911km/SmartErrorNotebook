use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAttachmentInput {
    pub question_id: String,
    #[serde(rename = "type", alias = "type_")]
    pub type_: String,
    pub file_type: String,
    pub base64_data: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpsertAttachmentInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    #[serde(default)]
    pub question_ids: Vec<String>,
    #[serde(default)]
    pub question_id: Option<String>,
    #[serde(rename = "type", alias = "type_")]
    pub attachment_type: String,
    pub file_type: String,
    pub base64_data: Vec<u8>,
    pub hash: String,
}
