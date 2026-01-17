// 创建附件表

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Attachments::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Attachments::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Attachments::QuestionId).string().not_null())
                    .col(ColumnDef::new(Attachments::Type).text().not_null())
                    .col(ColumnDef::new(Attachments::FileType).text().not_null())
                    .col(ColumnDef::new(Attachments::Path).string().not_null())
                    .col(ColumnDef::new(Attachments::Hash).string().not_null())
                    .col(ColumnDef::new(Attachments::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Attachments::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Attachments::DeletedAt).big_integer())
                    .col(ColumnDef::new(Attachments::Version).integer().not_null().default(0))
                    .col(ColumnDef::new(Attachments::SyncStatus).string().not_null().default("synced"))
                    .col(ColumnDef::new(Attachments::SyncHash).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_attachments_error_questions")
                            .from(Attachments::Table, Attachments::QuestionId)
                            .to(ErrorQuestions::Table, ErrorQuestions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Attachments::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Attachments {
    Table,
    Id,
    QuestionId,
    Type,
    FileType,
    Path,
    Hash,
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