// 出题来源表

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sources")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub question_id: String,
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
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
    #[sea_orm(
        belongs_to = "super::subject::Entity",
        from = "Column::SubjectId",
        to = "super::subject::Column::Id"
    )]
    Subject,
}

impl Related<super::error_question::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorQuestion.def()
    }
}

impl Related<super::subject::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subject.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}