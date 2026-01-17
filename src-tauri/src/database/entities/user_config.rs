// 用户配置表

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_config")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub username: String,
    pub email: String,
    pub student_num: String,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub theme: Option<String>,
    pub password_hash: Option<String>,
    pub ai_base_url: Option<String>,
    pub ai_key: Option<String>,
    pub sync: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}