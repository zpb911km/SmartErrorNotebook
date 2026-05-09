// 迁移：更新 SRS 数据表以支持连续反馈的 SDR 模型

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. 添加新字段

        // stability (稳定性，单位天)
        manager
            .alter_table(
                Table::alter()
                    .table(SrsData::Table)
                    .add_column_if_not_exists(ColumnDef::new(SrsData::Stability).float().not_null().default(3.0))
                    .to_owned(),
            )
            .await?;

        // next_review_at (下次复习时间戳)
        manager
            .alter_table(
                Table::alter()
                    .table(SrsData::Table)
                    .add_column_if_not_exists(ColumnDef::new(SrsData::NextReviewAt).big_integer().null())
                    .to_owned(),
            )
            .await?;

        // last_review_at (上次复习时间戳，改为 nullable) - 这里先不修改，等删除旧列后再添加
        // feedback_history (反馈历史 JSON)
        manager
            .alter_table(
                Table::alter()
                    .table(SrsData::Table)
                    .add_column_if_not_exists(ColumnDef::new(SrsData::FeedbackHistory).string().not_null().default("[]"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚：恢复原有字段结构（简单降级）
        manager
            .alter_table(
                Table::alter()
                    .table(SrsData::Table)
                    .drop_column(SrsData::Stability)
                    .drop_column(SrsData::NextReviewAt)
                    .drop_column(SrsData::FeedbackHistory)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum SrsData {
    Table,
    Stability,
    NextReviewAt,
    FeedbackHistory,
}
