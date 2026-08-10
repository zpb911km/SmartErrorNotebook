// SRS 数据相关命令 - 基于连续反馈的 SDR 模型

use crate::database::entities::srs_data;
use crate::repository::srs_data::{NewSrsData, SrsStateChanges, SyncedSrsData};
use crate::srs::{
    config, days_elapsed, predict_retrievability, review_card, update_feedback_history,
};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

// ==================== Input/Output Types ====================

/// 创建/初始化 SRS 数据的输入参数
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateSRSDataInput {
    /// 错题 ID
    pub question_id: String,
    /// 初始难度；未提供时使用 `config::INITIAL_DIFFICULTY`
    pub difficulty: Option<f32>,
}

/// 用于同步的 SRS UPSERT 输入
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpsertSRSDataInput {
    pub id: String,
    pub version: i32,
    pub status: String,
    pub deleted_at: Option<i64>,
    pub question_id: String,
    pub stability: f32,
    pub difficulty: f32,
    pub next_review_at: Option<i64>,
    pub lastreviewed_at: Option<i64>,
    pub review_count: i32,
    pub feedback_history: String,
}

/// 提交复习结果的输入参数
#[derive(serde::Serialize, serde::Deserialize)]
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

/// SRS 卡片信息输出
#[derive(serde::Serialize, serde::Deserialize)]
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
#[derive(serde::Serialize, serde::Deserialize)]
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
#[derive(serde::Serialize, serde::Deserialize)]
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

fn card(s: srs_data::Model, now: i64, is_due: bool) -> SRSCardOutput {
    SRSCardOutput {
        id: s.id,
        question_id: s.question_id,
        stability: s.stability,
        difficulty: s.difficulty,
        recall_rate: predict_retrievability(s.stability, days_elapsed(s.lastreviewed_at, now)),
        next_review_at: s.next_review_at,
        last_review_at: s.lastreviewed_at,
        review_count: s.review_count,
        is_due,
    }
}

// ==================== Command Functions ====================

/// 为指定题目创建或初始化 SRS 数据
#[tauri::command]
pub async fn create_srs_data(
    state: State<'_, AppState>,
    input: CreateSRSDataInput,
) -> Result<SRSCardOutput, String> {
    if state
        .repositories
        .srs_data
        .find_by_question(input.question_id.clone(), false)
        .await?
        .is_some()
    {
        return Err(format!("SrsData is exist: {}", input.question_id));
    }
    let now = chrono::Utc::now().timestamp();
    let model = state
        .repositories
        .srs_data
        .create(NewSrsData {
            id: Uuid::new_v4().to_string(),
            question_id: input.question_id,
            stability: config::INITIAL_STABILITY,
            difficulty: input.difficulty.unwrap_or(config::INITIAL_DIFFICULTY),
            next_review_at: Some(now + 24 * 3600),
            last_reviewed_at: Some(now),
            review_count: 1,
            feedback_history: "[]".into(),
            now,
        })
        .await?;
    Ok(card(model, now, false))
}

/// 获取待复习的题目列表
#[tauri::command]
pub async fn get_due_questions(
    state: State<'_, AppState>,
    limit: Option<i32>,
) -> Result<Vec<SRSCardOutput>, String> {
    let now = chrono::Utc::now().timestamp();
    let mut cards: Vec<_> = state
        .repositories
        .srs_data
        .list_active()
        .await?
        .into_iter()
        .filter(|s| s.next_review_at.map(|n| now >= n).unwrap_or(true))
        .collect();
    cards.sort_by(|a, b| a.stability.partial_cmp(&b.stability).unwrap());
    cards.truncate(limit.unwrap_or(1000) as usize);
    Ok(cards.into_iter().map(|s| card(s, now, true)).collect())
}

/// 提交复习结果并更新 SRS 状态
#[tauri::command]
pub async fn submit_review_result(
    state: State<'_, AppState>,
    input: SubmitReviewInput,
) -> Result<ReviewOutput, String> {
    let now = chrono::Utc::now().timestamp();
    let model = state
        .repositories
        .srs_data
        .find_by_question(input.question_id, false)
        .await?
        .ok_or_else(|| "SRS data not found".to_string())?;
    let result = review_card(&model, now, input.feedback)?;
    state
        .repositories
        .srs_data
        .update_state(SrsStateChanges {
            id: model.id,
            stability: result.new_stability,
            difficulty: result.new_difficulty,
            next_review_at: Some(result.next_review_at),
            last_reviewed_at: Some(now),
            review_count: model.review_count + 1,
            feedback_history: update_feedback_history(&model.feedback_history, input.feedback),
            deleted_at: None,
            now,
        })
        .await?;
    Ok(ReviewOutput {
        next_interval_days: result.next_interval_days,
        new_stability: result.new_stability,
        new_difficulty: result.new_difficulty,
        next_review_at: result.next_review_at,
    })
}

/// 获取单个题目的 SRS 状态
#[tauri::command]
pub async fn get_question_srs_status(
    state: State<'_, AppState>,
    question_id: String,
) -> Result<Option<SRSCardOutput>, String> {
    let now = chrono::Utc::now().timestamp();
    Ok(state
        .repositories
        .srs_data
        .find_by_question(question_id, true)
        .await?
        .map(|s| {
            let due = s.next_review_at.map(|n| n <= now).unwrap_or(true);
            card(s, now, due)
        }))
}

/// 将单个题目的 SRS 进度重置为新卡片状态
#[tauri::command]
pub async fn reset_srs_progress(
    state: State<'_, AppState>,
    question_id: String,
) -> Result<SRSCardOutput, String> {
    let now = chrono::Utc::now().timestamp();
    if let Some(model) = state
        .repositories
        .srs_data
        .find_by_question(question_id.clone(), true)
        .await?
    {
        let updated = state
            .repositories
            .srs_data
            .update_state(SrsStateChanges {
                id: model.id,
                stability: config::INITIAL_STABILITY,
                difficulty: config::INITIAL_DIFFICULTY,
                next_review_at: Some(now),
                last_reviewed_at: Some(now),
                review_count: 1,
                feedback_history: "[]".into(),
                deleted_at: model.deleted_at,
                now,
            })
            .await?;
        Ok(card(updated, now, true))
    } else {
        create_srs_data(
            state,
            CreateSRSDataInput {
                question_id,
                difficulty: None,
            },
        )
        .await
    }
}

// ==================== Tool/API Functions ====================

/// 获取当前待复习卡片总数
#[tauri::command]
pub async fn get_due_count(state: State<'_, AppState>) -> Result<i32, String> {
    let now = chrono::Utc::now().timestamp();
    Ok(state
        .repositories
        .srs_data
        .list_active()
        .await?
        .into_iter()
        .filter(|s| s.next_review_at.map(|n| now >= n).unwrap_or(true))
        .count() as i32)
}

/// 获取所有 SRS 数据的统计信息
#[tauri::command]
pub async fn get_srs_statistics(state: State<'_, AppState>) -> Result<SRSStatistics, String> {
    let all = state.repositories.srs_data.list_active().await?;
    if all.is_empty() {
        return Ok(SRSStatistics {
            total: 0,
            due_count: 0,
            new_cards: 0,
            avg_stability: 0.0,
            avg_difficulty: 0.0,
            total_reviews: 0,
        });
    }
    let now = chrono::Utc::now().timestamp();
    let total = all.len() as i32;
    let due_count = all
        .iter()
        .filter(|s| s.next_review_at.map(|n| now >= n).unwrap_or(true))
        .count() as i32;
    let new_cards = all.iter().filter(|s| s.review_count == 1).count() as i32;
    let avg_stability = all.iter().map(|s| s.stability as f64).sum::<f64>() / all.len() as f64;
    let avg_difficulty = all.iter().map(|s| s.difficulty as f64).sum::<f64>() / all.len() as f64;
    let total_reviews = all.iter().map(|s| s.review_count as i64).sum();
    Ok(SRSStatistics {
        total,
        due_count,
        new_cards,
        avg_stability: avg_stability as f32,
        avg_difficulty: avg_difficulty as f32,
        total_reviews,
    })
}

/// 获取所有卡片
#[tauri::command]
pub async fn get_all_cards(state: State<'_, AppState>) -> Result<Vec<SRSCardOutput>, String> {
    let now = chrono::Utc::now().timestamp();
    Ok(state
        .repositories
        .srs_data
        .list_active()
        .await?
        .into_iter()
        .map(|s| {
            let due = s.next_review_at.map(|n| n <= now).unwrap_or(true);
            card(s, now, due)
        })
        .collect())
}

/// 根据 ID 插入或更新同步下发的 SRS 数据
#[tauri::command]
pub async fn upsert_srs_data(
    state: State<'_, AppState>,
    input: UpsertSRSDataInput,
) -> Result<(), String> {
    state
        .repositories
        .srs_data
        .upsert_synced(SyncedSrsData {
            id: input.id,
            version: input.version,
            deleted_at: input.deleted_at,
            question_id: input.question_id,
            stability: input.stability,
            difficulty: input.difficulty,
            next_review_at: input.next_review_at,
            last_reviewed_at: input.lastreviewed_at,
            review_count: input.review_count,
            feedback_history: input.feedback_history,
            now: chrono::Utc::now().timestamp(),
        })
        .await
        .map_err(Into::into)
}
