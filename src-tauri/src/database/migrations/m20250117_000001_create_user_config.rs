// 创建用户配置表

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserConfig::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserConfig::Username)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserConfig::Email).string().not_null())
                    .col(ColumnDef::new(UserConfig::StudentNum).string().not_null())
                    .col(ColumnDef::new(UserConfig::Phone).string())
                    .col(ColumnDef::new(UserConfig::Avatar).string())
                    .col(ColumnDef::new(UserConfig::Theme).string())
                    .col(ColumnDef::new(UserConfig::PasswordHash).string())
                    .col(ColumnDef::new(UserConfig::AiBaseUrl).string())
                    .col(ColumnDef::new(UserConfig::AiKey).string())
                    .col(
                        ColumnDef::new(UserConfig::Sync)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(UserConfig::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserConfig::UpdatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserConfig::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserConfig {
    Table,
    Username,
    Email,
    StudentNum,
    Phone,
    Avatar,
    Theme,
    PasswordHash,
    AiBaseUrl,
    AiKey,
    Sync,
    CreatedAt,
    UpdatedAt,
}
