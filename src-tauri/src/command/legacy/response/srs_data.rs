use serde::{Deserialize, Serialize};

/// SRS 卡片信息输出
#[derive(Serialize, Deserialize)]
pub struct SRSCardOutput {
    pub id: String,
    pub question_id: String,
    pub stability: f32,
    pub difficulty: f32,
    pub recall_rate: f32,
    pub next_review_at: Option<i64>,
    pub last_review_at: Option<i64>,
    pub review_count: i32,
    pub is_due: bool,
}

/// 复习结果输出
#[derive(Serialize, Deserialize)]
pub struct ReviewOutput {
    /// 下次复习间隔（天）
    pub next_interval_days: f32,
    /// 更新后的稳定性
    pub new_stability: f32,
    /// 更新后的难度
    pub new_difficulty: f32,
    /// 建议的下次复习时间戳（秒）
    pub next_review_at: i64,
}

/// SRS 统计数据输出
#[derive(Serialize, Deserialize)]
pub struct SRSStatistics {
    /// 总卡片数
    pub total: i32,
    /// 待复习数量
    pub due_count: i32,
    /// 新卡片数量（review_count = 1）
    pub new_cards: i32,
    /// 平均稳定性（天）
    pub avg_stability: f32,
    /// 平均难度
    pub avg_difficulty: f32,
    /// 总复习次数
    pub total_reviews: i64,
}
