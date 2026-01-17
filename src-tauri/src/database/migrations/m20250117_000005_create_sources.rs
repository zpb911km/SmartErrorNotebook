// 创建出题来源表

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Sources::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Sources::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Sources::QuestionId).string().not_null())
                    .col(ColumnDef::new(Sources::SubjectId).string())
                    .col(ColumnDef::new(Sources::Book).string())
                    .col(ColumnDef::new(Sources::Chapter).string())
                    .col(ColumnDef::new(Sources::Knowledge).string())
                    .col(ColumnDef::new(Sources::CreatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Sources::UpdatedAt).big_integer().not_null())
                    .col(ColumnDef::new(Sources::DeletedAt).big_integer())
                    .col(ColumnDef::new(Sources::Version).integer().not_null().default(0))
                    .col(ColumnDef::new(Sources::SyncStatus).string().not_null().default("synced"))
                    .col(ColumnDef::new(Sources::SyncHash).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sources_error_questions")
                            .from(Sources::Table, Sources::QuestionId)
                            .to(ErrorQuestions::Table, ErrorQuestions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sources_subjects")
                            .from(Sources::Table, Sources::SubjectId)
                            .to(Subjects::Table, Subjects::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Sources::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Sources {
    Table,
    Id,
    QuestionId,
    SubjectId,
    Book,
    Chapter,
    Knowledge,
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

#[derive(DeriveIden)]
enum Subjects {
    Table,
    Id,
}