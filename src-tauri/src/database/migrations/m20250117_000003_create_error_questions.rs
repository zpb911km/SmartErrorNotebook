// 创建错题表

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ErrorQuestions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ErrorQuestions::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(ErrorQuestions::Userid).string().not_null())
                    .col(ColumnDef::new(ErrorQuestions::Subjectid).string().not_null())
                    .col(ColumnDef::new(ErrorQuestions::Prompt).string().not_null())
                    .col(ColumnDef::new(ErrorQuestions::Type).string().not_null())
                    .col(ColumnDef::new(ErrorQuestions::Answer).text())
                    .col(ColumnDef::new(ErrorQuestions::Analysis).text())
                    .col(ColumnDef::new(ErrorQuestions::ErrorNote).text())
                    .col(ColumnDef::new(ErrorQuestions::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(ErrorQuestions::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(ErrorQuestions::DeletedAt).big_integer())
                    .col(ColumnDef::new(ErrorQuestions::Version).integer().not_null().default(0))
                    .col(ColumnDef::new(ErrorQuestions::SyncStatus).string().not_null().default("synced"))
                    .col(ColumnDef::new(ErrorQuestions::SyncHash).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_error_questions_subjects")
                            .from(ErrorQuestions::Table, ErrorQuestions::Subjectid)
                            .to(Subjects::Table, Subjects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ErrorQuestions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ErrorQuestions {
    Table,
    Id,
    Userid,
    Subjectid,
    Prompt,
    Type,
    Answer,
    Analysis,
    ErrorNote,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Version,
    SyncStatus,
    SyncHash,
}

#[derive(DeriveIden)]
enum Subjects {
    Table,
    Id,
}