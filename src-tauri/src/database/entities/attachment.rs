// 附件表

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "attachments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub question_id: String,
    #[sea_orm(column_type = "Text")]
    pub type_: String,
    #[sea_orm(column_type = "Text")]
    pub file_type: String,
    pub base64_data: Vec<u8>,
    pub hash: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub version: i32,
    pub sync_status: String,
    pub sync_hash: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
