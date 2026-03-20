// 为 error_questions 表添加 source_id 列

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ErrorQuestions::Table)
                    .add_column(ColumnDef::new(ErrorQuestions::Sourceid).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ErrorQuestions::Table)
                    .drop_column(ErrorQuestions::Sourceid)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ErrorQuestions {
    Table,
    Sourceid,
}
