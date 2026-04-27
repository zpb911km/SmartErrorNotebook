// SRS 数据表 - 基于连续反馈的 SDR 记忆度量模型

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SRS 模型参数常量
pub mod config {
    /// 目标可提取度 (Target Retrievability)
    pub const TARGET_R: f32 = 0.85;

    /// 稳定性学习率
    pub const ETA_S: f32 = 0.3;

    /// 难度学习率
    pub const ETA_D: f32 = 0.05;

    /// 单次稳定性最大放大倍数
    pub const MAX_S_FACTOR: f32 = 5.0;

    /// 单次稳定性最小缩小比例
    pub const MIN_S_FACTOR: f32 = 0.5;

    /// 初始稳定性 (天) - 错题默认较低
    pub const INITIAL_STABILITY: f32 = 3.0;

    /// 初始难度 (适中)
    pub const INITIAL_DIFFICULTY: f32 = 5.0;

    /// 难度下限
    pub const MIN_DIFFICULTY: f32 = 1.0;

    /// 难度上限
    pub const MAX_DIFFICULTY: f32 = 10.0;

    /// 最大复习间隔 (天) - 超过视为永久记忆
    pub const MAX_INTERVAL_DAYS: i64 = 1000;

    /// 反馈历史记录长度
    pub const FEEDBACK_HISTORY_LEN: usize = 5;
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "srs_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,

    /// 关联的错题 ID
    pub question_id: String,

    // === SDR 核心参数 ===

    /// 稳定性 (Stability): 单位天，表示记忆强度
    /// 值越大，遗忘曲线越平缓，间隔越长
    /// 初始值为 3.0 天 (错题特性)
    pub stability: f32,

    /// 难度 (Difficulty): [1.0, 10.0]
    /// 值越大表示题目越难，稳定性增长越慢
    /// 根据用户反馈历史自适应调整
    pub difficulty: f32,

    /// 下次复习时间戳 (秒)
    /// None 表示新卡片或从未安排复习
    pub next_review_at: Option<i64>,

    /// 上次复习时间戳 (秒)
    /// None 表示尚未进行过复习
    pub last_review_at: Option<i64>,

    // === 统计信息 ===

    /// 总复习次数
    pub review_count: i32,

    /// 最近最多 5 次反馈记录 (JSON 数组，如 [0.8, 0.9, 0.7])
    pub feedback_history: String,

    // === 元数据 ===

    pub created_at: i64,
    pub updated_at: i64,
    pub version: i32,
    pub sync_status: String,
    pub sync_hash: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::error_question::Entity",
        from = "Column::QuestionId",
        to = "super::error_question::Column::Id"
    )]
    ErrorQuestion,
}

impl Related<super::error_question::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ErrorQuestion.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
