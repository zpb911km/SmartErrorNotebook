pub struct NewAttachment {
    pub id: String,
    pub question_id: String,
    pub type_: String,
    pub file_type: String,
    pub base64_data: String,
    pub now: i64,
}
pub struct SyncedAttachment {
    pub id: String,
    pub version: i32,
    pub deleted_at: Option<i64>,
    pub question_ids: Vec<String>,
    pub type_: String,
    pub file_type: String,
    pub base64_data: Vec<u8>,
    pub now: i64,
}
