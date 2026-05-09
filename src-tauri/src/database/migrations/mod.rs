// 数据库迁移模块

pub use sea_orm_migration::prelude::*;

mod m20250117_000001_create_user_config;
mod m20250117_000002_create_subjects;
mod m20250117_000003_create_error_questions;
mod m20250117_000004_create_srs_data;
mod m20250117_000006_create_error_tags;
mod m20250117_000007_create_attachments;
mod m20250130_000002_recreate_sources;
mod m20250130_000003_alter_attachments_base64;
mod m20250206_000001_add_sourceid_to_error_questions;
mod m20260428_000005_alter_srs_data_sdr_model;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250117_000001_create_user_config::Migration),
            Box::new(m20250117_000002_create_subjects::Migration),
            Box::new(m20250117_000003_create_error_questions::Migration),
            Box::new(m20250117_000004_create_srs_data::Migration),
            Box::new(m20250117_000006_create_error_tags::Migration),
            Box::new(m20250117_000007_create_attachments::Migration),
            Box::new(m20250130_000002_recreate_sources::Migration),
            Box::new(m20250130_000003_alter_attachments_base64::Migration),
            Box::new(m20250206_000001_add_sourceid_to_error_questions::Migration),
            Box::new(m20260428_000005_alter_srs_data_sdr_model::Migration),
        ]
    }
}
