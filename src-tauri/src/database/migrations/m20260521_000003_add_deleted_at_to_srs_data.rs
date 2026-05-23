// 为 srs_data 表添加 deleted_at 字段，支持级联软删除

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SrsData::Table)
                    .add_column(ColumnDef::new(SrsData::DeletedAt).big_integer().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SrsData::Table)
                    .drop_column(SrsData::DeletedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum SrsData {
    Table,
    DeletedAt,
}
