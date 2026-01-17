// 创建错题标签表

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ErrorTags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ErrorTags::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(ErrorTags::QuestionId).string().not_null())
                    .col(ColumnDef::new(ErrorTags::Name).string().not_null())
                    .col(ColumnDef::new(ErrorTags::Color).string().not_null())
                    .col(ColumnDef::new(ErrorTags::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(ErrorTags::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(ErrorTags::DeletedAt).big_integer())
                    .col(ColumnDef::new(ErrorTags::Version).integer().not_null().default(0))
                    .col(ColumnDef::new(ErrorTags::SyncStatus).string().not_null().default("synced"))
                    .col(ColumnDef::new(ErrorTags::SyncHash).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_error_tags_error_questions")
                            .from(ErrorTags::Table, ErrorTags::QuestionId)
                            .to(ErrorQuestions::Table, ErrorQuestions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ErrorTags::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ErrorTags {
    Table,
    Id,
    QuestionId,
    Name,
    Color,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Version,
    SyncStatus,
    SyncHash,
}

#[derive(DeriveIden)]
enum ErrorQuestions {
    Table,
    Id,
}