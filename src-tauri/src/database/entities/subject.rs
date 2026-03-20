// 科目表

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "subjects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub version: i32,
    pub sync_status: String,
    pub sync_hash: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::error_question::Entity")]
    ErrorQuestions,
    #[sea_orm(has_many = "super::source::Entity")]
    Sources,
}

impl Related<super::error_question::Entity> for Entity {
    fn to() -> RelationDef {
        super::error_question::Relation::Subject.def()
    }
}

impl Related<super::source::Entity> for Entity {
    fn to() -> RelationDef {
        super::source::Relation::Subject.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
