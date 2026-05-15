// SRS 数据表 - 基于连续反馈的 SDR 记忆度量模型

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub lastreviewed_at: Option<i64>,

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
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
