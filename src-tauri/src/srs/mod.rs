// FSRS-5 核心算法模块 - 连续反馈版
//
// 基于 Free Spaced Repetition Scheduler 5.0 的数学公式，
// 将离散 4 级评分 (Again/Hard/Good/Easy) 映射到连续反馈 f ∈ [0, 1]，
// 保留全部外部接口签名不变，srs_data.rs / 前端零改动。
//
// 参考文献: https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm

use crate::database::entities::srs_data;
use serde::{Deserialize, Serialize};

/// FSRS-5 模型参数常量
pub mod config {
    // ═══════════════════════════════════════════════════════════
    // FSRS-5 官方默认参数 (19 个可优化权重)
    // 来源: https://github.com/open-spaced-repetition/fsrs4anki
    // ═══════════════════════════════════════════════════════════
    //
    // 索引说明:
    //   w0-w3: 初始稳定性 (Again/Hard/Good/Easy)
    //   w4-w5: 初始难度
    //   w6-w7: 难度更新
    //   w8-w10: 成功复习稳定性增长
    //   w11-w14: 遗忘后稳定性
    //   w15-w16: 评分乘数 (Hard惩罚 / Easy奖励)
    //   w17-w18: 同日复习
    pub const W: [f32; 19] = [
        0.40255, 1.18385, 3.173, 15.69105, 7.1949, 0.5345, 1.4604, 0.0046,
        1.54575, 0.1192, 1.01925, 1.9395, 0.11, 0.29605, 2.2698, 0.2315,
        2.9898, 0.51655, 0.6621,
    ];

    // ── 向后兼容常量 ──────────────────────────────────────
    // 旧代码引用: create_srs_data / reset_srs_progress
    // 映射到 FSRS 的 Good 级初始值
    pub const INITIAL_STABILITY: f32 = W[2]; // = 3.173
    /// 初始难度，对应 FSRS-5 D₀(Good) = w₄ - w₅·(3-3) = w₄
    pub const INITIAL_DIFFICULTY: f32 = W[4]; // = 7.1949

    // ── FSRS 配置 ─────────────────────────────────────────
    /// 最大间隔天数（超过视为永久记忆）
    pub const MAX_INTERVAL_DAYS: i64 = 1000;

    /// 目标保持率（默认 90%，此时 interval ≈ S）
    pub const TARGET_RETENTION: f64 = 0.9;

    /// 反馈历史长度（仅用于展示，不再参与 D 更新）
    pub const FEEDBACK_HISTORY_LEN: usize = 5;

    /// 遗忘判定阈值
    /// f < LAPSE_THRESHOLD → 进入遗忘路径 (post-lapse stability)
    /// f ≥ LAPSE_THRESHOLD → 进入成功路径
    pub const LAPSE_THRESHOLD: f32 = 0.2;
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

// ═══════════════════════════════════════════════════════════════
// 公共接口 (签名保持不变)
// ═══════════════════════════════════════════════════════════════

/// 计算从上次复习到现在经过的天数
pub fn days_elapsed(last_review_at: Option<i64>, now: i64) -> f64 {
    match last_review_at {
        Some(lr) => ((now - lr) as f64) / (24.0 * 3600.0),
        None => 0.0,
    }
}

/// 预测当前可提取度 R（FSRS 幂函数遗忘曲线）
///
/// $$R(t,S) = \left(1 + \frac{19}{81} \cdot \frac{t}{S}\right)^{-0.5}$$
///
/// 当 t = S 时，R = 0.9（90% 保持率），这比指数曲线更符合人类记忆特征。
pub fn predict_retrievability(stability: f32, elapsed_days: f64) -> f32 {
    if stability <= 0.0 {
        return 0.0;
    }
    let t_over_s = elapsed_days / stability as f64;
    (1.0 + (19.0 / 81.0) * t_over_s).powf(-0.5) as f32
}

/// 计算下次复习间隔（FSRS 公式）
///
/// $$I(r,S) = S \cdot \frac{81}{19} \cdot \left(r^{-2} - 1\right)$$
///
/// 当 r = 0.9 时，I ≈ S（因为 t=S 时 R 恰好等于 0.9）。
pub fn compute_next_interval(stability: f32) -> i64 {
    if stability <= 0.0 {
        return 1;
    }
    let r = config::TARGET_RETENTION;
    let interval_days = stability as f64 * (81.0 / 19.0) * (r.powf(-2.0) - 1.0);
    let interval = interval_days.max(1.0).ceil() as i64;
    interval.min(config::MAX_INTERVAL_DAYS)
}

/// 维护反馈历史：添加新反馈，保持最多 FEEDBACK_HISTORY_LEN 条
pub fn update_feedback_history(history_json: &str, new_feedback: f32) -> String {
    let mut history: Vec<f32> = serde_json::from_str(history_json).unwrap_or_default();
    history.push(new_feedback);
    if history.len() > config::FEEDBACK_HISTORY_LEN {
        let excess = history.len() - config::FEEDBACK_HISTORY_LEN;
        history.drain(0..excess);
    }
    serde_json::to_string(&history).unwrap_or_else(|_| "[]".to_string())
}

// ═══════════════════════════════════════════════════════════════
// 核心算法：review_card
// ═══════════════════════════════════════════════════════════════

/// 核心算法：处理一次复习并计算下次复习间隔（FSRS-5 连续反馈版）
///
/// # 参数
/// - `card`: 当前卡片状态
/// - `now`: 当前 Unix 时间戳（秒）
/// - `feedback`: 用户反馈值 [0, 1]
///   - 0.0: 完全忘记 → 重置为新卡片
///   - 1.0: 完全熟记 → 最大间隔
///   - (0, 0.2): 遗忘路径（post-lapse stability）
///   - [0.2, 1.0): 成功路径（稳定性增长）
///
/// # 连续反馈 → FSRS Grade 映射
/// $$G = 3f + 1, \quad G \in [1, 4]$$
///
/// 等价于离散 FSRS 的 Again(1) / Hard(2) / Good(3) / Easy(4)。
pub fn review_card(card: &srs_data::Model, now: i64, feedback: f32) -> Result<ReviewResult, String> {
    if feedback < 0.0 || feedback > 1.0 {
        return Err(format!("Feedback must be in [0, 1], got {}", feedback));
    }

    // ── 特殊边界处理 ──────────────────────────────────────
    if feedback == 0.0 {
        // 完全忘记 → 重置为新卡片
        return Ok(ReviewResult {
            next_interval_days: 1.0,
            new_stability: config::INITIAL_STABILITY,
            new_difficulty: config::INITIAL_DIFFICULTY,
            next_review_at: now,
        });
    }

    if feedback == 1.0 {
        // 完全熟记 → 最大间隔（视为永久记忆）
        let max_sec = (config::MAX_INTERVAL_DAYS as i64) * 24 * 3600;
        return Ok(ReviewResult {
            next_interval_days: config::MAX_INTERVAL_DAYS as f32,
            new_stability: card.stability * 5.0,
            new_difficulty: card.difficulty,
            next_review_at: now + max_sec,
        });
    }

    // ── 连续反馈 → FSRS Grade ────────────────────────────
    // G = 3f + 1,  G ∈ [1, 4]
    let grade = 3.0 * feedback + 1.0;

    // 距离上次复习的天数
    let elapsed_days = days_elapsed(card.lastreviewed_at, now);

    // 当前可提取度 R（FSRS 遗忘曲线）
    let r_pred = predict_retrievability(card.stability, elapsed_days);

    // ── 1. 难度更新（线性阻尼 + 均值回归）─────────────
    // ΔD = -w₆ · (G - 3)
    let delta_d = -config::W[6] * (grade - 3.0);
    // 线性阻尼：难度越高，变化越慢
    let d_damped = card.difficulty + delta_d * (10.0 - card.difficulty) / 9.0;
    // 均值回归：向 D₀(Easy) 靠拢，防止难度地狱
    let d0_easy = initial_difficulty(4.0); // D₀(4)
    let d_new = config::W[7] * d0_easy + (1.0 - config::W[7]) * d_damped;
    let d_new = clamp(d_new, 1.0, 10.0);

    // ── 2. 稳定性更新 ──────────────────────────────────
    let s_new = if feedback < config::LAPSE_THRESHOLD {
        // ── 遗忘路径（Post-lapse Stability）───────────
        // S_f' = w₁₁ · D^(-w₁₂) · [(S+1)^w₁₃ - 1] · e^(w₁₄ · (1-R))
        config::W[11]
            * card.difficulty.powf(-config::W[12])
            * ((card.stability + 1.0).powf(config::W[13]) - 1.0)
            * (config::W[14] * (1.0 - r_pred)).exp()
    } else {
        // ── 成功路径 ───────────────────────────────────
        // 连续反馈评分乘数 M
        let m = interpolate_multiplier(grade);
        // S_r' = S · [ e^w₈ · (11-D) · S^(-w₉) · (e^(w₁₀·(1-R)) - 1) · M + 1 ]
        card.stability
            * (config::W[8].exp()
                * (11.0 - d_new)
                * card.stability.powf(-config::W[9])
                * ((config::W[10] * (1.0 - r_pred)).exp() - 1.0)
                * m
                + 1.0)
    };

    // 稳定性保护（不能低于最小值）
    let s_new = s_new.max(0.01);

    // ── 3. 同日复习检测 ──────────────────────────────
    // 如果距上次复习 < 1 天，适用同日复习公式
    let s_final = if elapsed_days < 1.0 {
        // S'(S,G) = S · e^(w₁₇ · (G - 3 + w₁₈))
        s_new * (config::W[17] * (grade - 3.0 + config::W[18])).exp()
    } else {
        s_new
    };

    // 稳定性上限保护
    let s_final = s_final.min(36500.0); // 不超过 100 年

    // ── 4. 计算下次复习间隔 ──────────────────────────
    let next_interval_days = compute_next_interval(s_final);
    let next_review_at = now + (next_interval_days as i64) * 24 * 3600;

    Ok(ReviewResult {
        next_interval_days: next_interval_days as f32,
        new_stability: s_final,
        new_difficulty: d_new,
        next_review_at,
    })
}

// ═══════════════════════════════════════════════════════════════
// 内部辅助函数
// ═══════════════════════════════════════════════════════════════

/// 计算 FSRS 初始难度 D₀(G)
///
/// $$D_0(G) = w_4 - e^{w_5 \cdot (G - 1)} + 1$$
fn initial_difficulty(grade: f32) -> f32 {
    (config::W[4] - (config::W[5] * (grade - 1.0)).exp() + 1.0)
        .clamp(1.0, 10.0)
}

/// 连续反馈评分乘数 M(G)
///
/// 在离散 FSRS 中：
///   G=2 (Hard) → M = w₁₅ (惩罚)
///   G=3 (Good) → M = 1.0
///   G=4 (Easy) → M = w₁₆ (奖励)
///
/// 连续化：线性插值
fn interpolate_multiplier(grade: f32) -> f32 {
    let g = grade.clamp(2.0, 4.0);
    if g <= 3.0 {
        // G ∈ [2, 3]: w₁₅ → 1.0
        let t = (g - 2.0) / 1.0; // 0..=1
        config::W[15] + (1.0 - config::W[15]) * t
    } else {
        // G ∈ (3, 4]: 1.0 → w₁₆
        let t = (g - 3.0) / 1.0; // 0..=1
        1.0 + (config::W[16] - 1.0) * t
    }
}

/// 限制数值在 [low, high] 区间
fn clamp(value: f32, low: f32, high: f32) -> f32 {
    value.max(low).min(high)
}

// ═══════════════════════════════════════════════════════════════
// 单元测试
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建一个模拟卡片用于测试
    fn mock_card(
        stability: f32,
        difficulty: f32,
        last_reviewed_at: Option<i64>,
        review_count: i32,
        feedback_history: &str,
    ) -> srs_data::Model {
        srs_data::Model {
            id: "test".to_string(),
            question_id: "test-q".to_string(),
            stability,
            difficulty,
            next_review_at: None,
            lastreviewed_at: last_reviewed_at,
            review_count,
            feedback_history: feedback_history.to_string(),
            created_at: 0,
            updated_at: 0,
            version: 0,
            sync_status: "synced".to_string(),
            sync_hash: None,
            deleted_at: None,
        }
    }

    #[test]
    fn test_predict_retrievability_fsrs() {
        // 当 t = S 时，R 应为 0.9
        let r = predict_retrievability(10.0, 10.0);
        assert!((r - 0.9).abs() < 0.001, "R={} should be ≈0.9", r);

        // 当 t = 0 时，R 应为 1.0
        let r = predict_retrievability(10.0, 0.0);
        assert!((r - 1.0).abs() < 0.001, "R={} should be 1.0", r);

        // 当 t >> S 时，R → 0（幂函数肥尾，不会严格归零）
        let r = predict_retrievability(10.0, 1e8);
        assert!(r < 0.001, "R={} should be near 0", r);
    }

    #[test]
    fn test_initial_difficulty() {
        let d4 = initial_difficulty(4.0);
        assert!(d4 >= 1.0 && d4 <= 10.0, "D₀(4)={} out of range", d4);
        let d1 = initial_difficulty(1.0);
        assert!(d1 >= 1.0 && d1 <= 10.0, "D₀(1)={} out of range", d1);
    }

    #[test]
    fn test_compute_next_interval_fsrs() {
        // 当 r=0.9 时, I ≈ S
        let interval = compute_next_interval(10.0);
        assert!(
            (interval as f64 - 10.0).abs() < 1.0,
            "I={} should be ≈10 when S=10",
            interval
        );
    }

    #[test]
    fn test_review_card_good_feedback() {
        // f=0.7（接近 Good），预测应增长稳定性
        let card = mock_card(5.0, 5.0, Some(1000000), 5, "[0.6,0.7,0.65,0.7,0.6]");
        let now = 1000000 + 5 * 24 * 3600; // 5天后
        let result = review_card(&card, now, 0.7).unwrap();
        assert!(
            result.new_stability > card.stability,
            "S_new={} should be > S={} for good feedback",
            result.new_stability,
            card.stability
        );
        assert!(result.next_interval_days > 0.0);
        assert!(result.next_review_at > now);
    }

    #[test]
    fn test_review_card_lapse() {
        // f=0.1（遗忘），应大幅降低稳定性
        let card = mock_card(10.0, 5.0, Some(1000000), 5, "[0.5,0.6,0.7,0.6,0.5]");
        let now = 1000000 + 30 * 24 * 3600; // 30天后
        let result = review_card(&card, now, 0.1).unwrap();
        assert!(
            result.new_stability < card.stability,
            "S_new={} should be < S={} for lapse",
            result.new_stability,
            card.stability
        );
    }

    #[test]
    fn test_review_card_reset() {
        let card = mock_card(50.0, 3.0, Some(1000000), 20, "[]");
        let now = 2000000;
        let result = review_card(&card, now, 0.0).unwrap();
        assert_eq!(result.new_stability, config::INITIAL_STABILITY);
        assert_eq!(result.new_difficulty, config::INITIAL_DIFFICULTY);
    }

    #[test]
    fn test_review_card_max() {
        let card = mock_card(3.0, 5.0, Some(1000000), 1, "[]");
        let now = 1000000 + 24 * 3600;
        let result = review_card(&card, now, 1.0).unwrap();
        assert!(result.next_interval_days >= 999.0);
    }

    #[test]
    fn test_boundary_conditions() {
        let card = mock_card(3.0, 5.0, Some(1000000), 1, "[]");
        let now = 1000000 + 24 * 3600;
        // 不应 panic
        let _ = review_card(&card, now, 0.0).unwrap();
        let _ = review_card(&card, now, 0.2).unwrap();
        let _ = review_card(&card, now, 0.5).unwrap();
        let _ = review_card(&card, now, 1.0).unwrap();
    }

    #[test]
    fn test_update_feedback_history() {
        let h = update_feedback_history("[]", 0.5);
        assert_eq!(h, "[0.5]");
        let mut h = String::from("[]");
        for i in 0..10 {
            h = update_feedback_history(&h, (i as f32) / 10.0);
        }
        // 应该只保留最近 5 条
        let parsed: Vec<f32> = serde_json::from_str(&h).unwrap();
        assert_eq!(parsed.len(), config::FEEDBACK_HISTORY_LEN);
    }

    #[test]
    fn test_interpolate_multiplier() {
        let m_hard = interpolate_multiplier(2.0);
        assert!((m_hard - config::W[15]).abs() < 0.001, "M(2)={} ≠ w15={}", m_hard, config::W[15]);

        let m_good = interpolate_multiplier(3.0);
        assert!((m_good - 1.0).abs() < 0.001, "M(3)={} ≠ 1.0", m_good);

        let m_easy = interpolate_multiplier(4.0);
        assert!((m_easy - config::W[16]).abs() < 0.001, "M(4)={} ≠ w16={}", m_easy, config::W[16]);

        // 中点测试
        let m_mid = interpolate_multiplier(2.5);
        let expected = (config::W[15] + 1.0) / 2.0;
        assert!((m_mid - expected).abs() < 0.001, "M(2.5)={} ≠ mid={}", m_mid, expected);
    }
}
