use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SyncRecordHeader {
    pub id: String,
    pub table_name: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub updated_at: i64,
    pub created_at: i64,
}
#[derive(Debug, Clone, Serialize)]
pub struct SyncRecordOutput {
    pub id: String,
    pub table_name: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub updated_at: i64,
    #[serde(rename = "data")]
    pub data: serde_json::Value,
}
#[derive(Debug, Serialize)]
pub struct CascadeOrphanCheckResult {
    pub orphan_records_soft_deleted: Vec<String>,
    pub total_checked: usize,
}
