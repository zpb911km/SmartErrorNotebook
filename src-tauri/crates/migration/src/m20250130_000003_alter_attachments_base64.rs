// 修改attachments表，将path字段改为base64_data

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除旧的path列
        manager
            .alter_table(
                Table::alter()
                    .table(Attachments::Table)
                    .drop_column(Attachments::Path)
                    .to_owned(),
            )
            .await?;

        // 添加新的base64_data列
        manager
            .alter_table(
                Table::alter()
                    .table(Attachments::Table)
                    .add_column(ColumnDef::new(Attachments::Base64Data).text().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 撤销：删除base64_data列
        manager
            .alter_table(
                Table::alter()
                    .table(Attachments::Table)
                    .drop_column(Attachments::Base64Data)
                    .to_owned(),
            )
            .await?;

        // 添加回旧的path列
        manager
            .alter_table(
                Table::alter()
                    .table(Attachments::Table)
                    .add_column(ColumnDef::new(Attachments::Path).string().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Attachments {
    Table,
    Path,
    Base64Data,
}
