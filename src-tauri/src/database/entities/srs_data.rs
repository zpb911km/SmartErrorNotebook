// SRS 数据表

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "srs_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub question_id: String,
    pub difficulty: f32,
    pub mastery: f32,
    pub lastreviewed_at: i64,
    pub review_count: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub version: i32,
    pub sync_status: String,
    pub sync_hash: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::error_question::Entity",
        from = "Column::QuestionId",
        to = "super::error_question::Column::Id"
    )]
    ErrorQuestion,
}

impl Related<super::error_question::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorQuestion.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}