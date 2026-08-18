use serde::{Deserialize, Serialize};

/// 创建/初始化 SRS 数据的输入参数
#[derive(Serialize, Deserialize)]
pub struct CreateSRSDataInput {
    /// 错题 ID
    pub question_id: String,
    /// 初始难度；未提供时使用 `config::INITIAL_DIFFICULTY`
    pub difficulty: Option<f32>,
}

/// 用于同步的 SRS UPSERT 输入
#[derive(Serialize, Deserialize)]
pub struct UpsertSRSDataInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub question_id: String,
    pub stability: f32,
    pub difficulty: f32,
    pub next_review_at: Option<i64>,
    #[serde(alias = "lastreviewed_at")]
    pub last_review_at: Option<i64>,
    pub review_count: i32,
    pub feedback_history: String,
}

/// 提交复习结果的输入参数
#[derive(Serialize, Deserialize)]
pub struct SubmitReviewInput {
    /// 错题 ID
    pub question_id: String,
    /// 连续反馈值 [0, 1]：
    /// - [0.0, 0.2) → Again
    /// - [0.2, 0.6) → Hard
    /// - [0.6, 0.9) → Good
    /// - [0.9, 1.0] → Easy
    pub feedback: f32,
}
