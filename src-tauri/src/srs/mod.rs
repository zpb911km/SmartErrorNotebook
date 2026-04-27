// SRS 核心算法模块 - 基于连续反馈的 SDR 记忆度量模型

use crate::database::entities::srs_data;
use sea_orm::Set;
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

/// SRS 复习会话结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    /// 下次复习间隔 (天)
    pub next_interval_days: f32,
    /// 更新后的稳定性
    pub new_stability: f32,
    /// 更新后的难度
    pub new_difficulty: f32,
    /// 建议的下次复习时间戳 (秒)
    pub next_review_at: i64,
}

/// 待复习卡片信息
#[derive(Debug, Clone, Serialize)]
pub struct DueCard {
    pub id: String,
    pub question_id: String,
    pub stability: f32,
    pub difficulty: f32,
    pub last_review_at: Option<i64>,
    pub review_count: i32,
}

/// 计算从上次复习到现在经过的天数
pub fn days_elapsed(last_review_at: Option<i64>, now: i64) -> f64 {
    match last_review_at {
        Some(lr) => ((now - lr) as f64) / (24.0 * 3600.0),
        None => f64::MAX,
    }
}

/// 预测当前可提取度 R = exp(-elapsed_days / stability)
/// R ∈ (0, 1], 1 表示完全记得，接近 0 表示几乎遗忘
pub fn predict_retrievability(stability: f32, elapsed_days: f64) -> f32 {
    if stability <= 0.0 {
        return 0.0;
    }
    (-elapsed_days / stability as f64).exp() as f32
}

/// 根据目标可提取度和当前稳定性计算下次复习间隔 (天)
/// interval = -stability * ln(target_R)
pub fn compute_next_interval(stability: f32) -> i64 {
    if stability <= 0.0 {
        return 1; // 至少 1 天后
    }
    let interval = -stability as f64 * (config::TARGET_R as f64).ln();
    let interval_days = interval.max(1.0) as i64; // 至少 1 天
    // 应用最大间隔限制
    interval_days.min(config::MAX_INTERVAL_DAYS)
}

/// 限制数值在 [low, high] 区间
fn clamp(value: f32, low: f32, high: f32) -> f32 {
    value.max(low).min(high)
}

/// 维护反馈历史：添加新反馈，保持最多 FEEDBACK_HISTORY_LEN 个
pub fn update_feedback_history(history_json: &str, new_feedback: f32) -> String {
    let mut history: Vec<f32> = serde_json::from_str(history_json)
        .unwrap_or_else(|_| vec![]);

    history.push(new_feedback);

    // 只保留最近 N 条
    if history.len() > config::FEEDBACK_HISTORY_LEN {
        history.drain(0..history.len() - config::FEEDBACK_HISTORY_LEN);
    }

    serde_json::to_string(&history).unwrap_or_else(|_| "[]".to_string())
}

/// 计算反馈历史的平均值
fn avg_feedback_history(history_json: &str) -> f32 {
    let history: Vec<f32> = serde_json::from_str(history_json)
        .unwrap_or_else(|_| vec![]);

    if history.is_empty() {
        return 0.5; // 默认中等反馈
    }

    history.iter().sum::<f32>() / history.len() as f32
}

/// 核心算法：处理一次复习并计算下次复习间隔
///
/// # 参数
/// - `card`: 当前卡片状态
/// - `now`: 当前 Unix 时间戳 (秒)
/// - `feedback`: 用户反馈 [0, 1]
///   - 0: 清除所有 SRS 状态，当作新卡片
///   - 1: 视为熟记，设置最大间隔
///   - (0, 1): 连续反馈值
///
/// # 返回
/// ReviewResult 包含更新后的状态
pub fn review_card(card: &srs_data::Model, now: i64, feedback: f32) -> Result<ReviewResult, String> {
    // 边界检查
    if feedback < 0.0 || feedback > 1.0 {
        return Err(format!("Feedback must be in [0, 1], got {}", feedback));
    }

    // 特殊反馈处理
    if feedback == 0.0 {
        // 清除所有 SRS 状态，当作新卡片
        return Ok(ReviewResult {
            next_interval_days: 1.0,
            new_stability: config::INITIAL_STABILITY,
            new_difficulty: config::INITIAL_DIFFICULTY,
            next_review_at: now, // 立即复习
        });
    }

    if feedback == 1.0 {
        // 视为熟记，设置最大间隔
        return Ok(ReviewResult {
            next_interval_days: config::MAX_INTERVAL_DAYS as f32,
            new_stability: card.stability * config::MAX_S_FACTOR, // 最大化增长
            new_difficulty: card.difficulty, // 难度不变
            next_review_at: now + (config::MAX_INTERVAL_DAYS as i64 * 24 * 3600),
        });
    }

    // === 普通反馈处理 ===

    let elapsed_days = days_elapsed(card.last_review_at, now);
    let r_pred = predict_retrievability(card.stability, elapsed_days);
    let delta = feedback - r_pred;

    // 1. 更新稳定性 S
    // 因子受难度影响：简单卡片 (D 小) 对反馈更敏感
    let sensitivity = 10.0 / card.difficulty;
    let factor = 1.0 + config::ETA_S * delta * sensitivity;
    let clamped_factor = clamp(factor, config::MIN_S_FACTOR, config::MAX_S_FACTOR);
    let new_stability = card.stability * clamped_factor;

    // 2. 更新难度 D (慢变，基于历史平均反馈)
    let new_history = update_feedback_history(&card.feedback_history, feedback);
    let _new_history = new_history; // unused warning fix
    let avg_f = avg_feedback_history(&_new_history);
    let delta_d = 0.5 - avg_f; // 反馈 > 0.5 降低难度，< 0.5 增加难度
    let new_difficulty = clamp(card.difficulty + config::ETA_D * delta_d,
                               config::MIN_DIFFICULTY,
                               config::MAX_DIFFICULTY);

    // 3. 计算下次复习间隔
    let next_interval_days = compute_next_interval(new_stability);
    let next_review_at = now + (next_interval_days * 24 * 3600);

    Ok(ReviewResult {
        next_interval_days: next_interval_days as f32,
        new_stability,
        new_difficulty,
        next_review_at,
    })
}

/// 判断卡片是否应该被复习 (到期)
pub fn is_due(card: &srs_data::Model, now: i64) -> bool {
    match card.next_review_at {
        None => true, // 未安排复习，视为待复习
        Some(next) => now >= next,
    }
}

/// 初始化一个新的 SRS 记录 (第一次复习)
pub fn initialize_srs(now: i64) -> srs_data::ActiveModel {
    let now_ts = now;

    srs_data::ActiveModel {
        id: Default::default(),
        question_id: Default::default(),
        stability: Set(config::INITIAL_STABILITY),
        difficulty: Set(config::INITIAL_DIFFICULTY),
        next_review_at: Set(Some(now_ts + 1 * 24 * 3600)), // 默认 1 天后
        last_review_at: Set(Some(now_ts)),
        review_count: Set(1),
        feedback_history: Set("[]".to_string()),
        created_at: Set(now_ts),
        updated_at: Set(now_ts),
        version: Default::default(),
        sync_status: Default::default(),
        sync_hash: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predict_retrievability() {
        // S=3 天时，3 天后 R ≈ e^-1 ≈ 0.368
        let r = predict_retrievability(3.0, 3.0);
        assert!((r - 0.368).abs() < 0.01);

        // S=10 天时，5 天后 R ≈ e^-0.5 ≈ 0.607
        let r = predict_retrievability(10.0, 5.0);
        assert!((r - 0.607).abs() < 0.01);
    }

    #[test]
    fn test_compute_interval() {
        // TARGET_R = 0.85, ln(0.85) ≈ -0.1625
        // S=3 时，interval ≈ 3 * 0.1625 ≈ 0.49 → 1 天 (最小)
        let interval = compute_next_interval(3.0);
        assert_eq!(interval, 1);

        // S=10 时，interval ≈ 10 * 0.1625 ≈ 1.63 → 2 天
        let interval = compute_next_interval(10.0);
        assert_eq!(interval, 2);
    }

    #[test]
    fn test_is_due() {
        let now = 1000000;

        // 没有 next_review_at 视为到期
        let card = srs_data::Model {
            id: "test".to_string(),
            question_id: "q1".to_string(),
            stability: 3.0,
            difficulty: 5.0,
            next_review_at: None,
            last_review_at: Some(now - 100),
            review_count: 1,
            feedback_history: "[]".to_string(),
            created_at: now,
            updated_at: now,
            version: 0,
            sync_status: "synced".to_string(),
            sync_hash: None,
        };
        assert!(is_due(&card, now));

        // 未来时间未到
        let card = srs_data::Model {
            next_review_at: Some(now + 1000),
            ..card.clone()
        };
        assert!(!is_due(&card, now));

        // 已过时间
        let card = srs_data::Model {
            next_review_at: Some(now - 1000),
            ..card.clone()
        };
        assert!(is_due(&card, now));
    }
}
