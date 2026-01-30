// 错题表

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "error_questions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub userid: String,
    pub subjectid: String,
    pub sourceid: Option<String>,
    pub prompt: String,
    pub type_: String,
    #[sea_orm(column_type = "Text")]
    pub answer: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub analysis: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub error_note: Option<String>,
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
        belongs_to = "super::subject::Entity",
        from = "Column::Subjectid",
        to = "super::subject::Column::Id"
    )]
    Subject,
    #[sea_orm(has_one = "super::srs_data::Entity")]
    SrsData,
    #[sea_orm(has_many = "super::error_tag::Entity")]
    ErrorTags,
    #[sea_orm(has_many = "super::attachment::Entity")]
    Attachments,
}

impl Related<super::subject::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subject.def()
    }
}

impl Related<super::srs_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SrsData.def()
    }
}

impl Related<super::error_tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorTags.def()
    }
}

impl Related<super::attachment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attachments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}