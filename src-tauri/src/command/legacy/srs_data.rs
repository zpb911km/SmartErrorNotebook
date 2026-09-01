// SRS 数据相关命令 - 基于连续反馈的 SDR 模型

use super::request::srs_data::{CreateSRSDataInput, SubmitReviewInput, UpsertSRSDataInput};
use super::response::srs_data::{ReviewOutput, SRSCardOutput, SRSStatistics};
use crate::model::SrsData;
use crate::repository::legacy::repository_model::srs_data::{
    NewSrsData, SrsStateChanges, SyncedSrsData,
};
use crate::repository::legacy::SrsDataRepository;
use crate::repository::{RepositoryFactory, RepositoryTransactionExecutor};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

fn card(s: SrsData, now: i64, is_due: bool) -> SRSCardOutput {
    SRSCardOutput {
        id: s.question_id.to_string(),
        question_id: s.question_id.to_string(),
        stability: s.stability(),
        difficulty: s.difficulty(),
        recall_rate: s.retrievability_at(now),
        next_review_at: s.next_review_at().map(|value| value.timestamp()),
        last_review_at: s.last_review_at().map(|value| value.timestamp()),
        review_count: s.review_count().clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        is_due,
    }
}

/// 为指定题目创建或初始化 SRS 数据
#[tauri::command]
pub async fn create_srs_data(
    state: State<'_, AppState>,
    input: CreateSRSDataInput,
) -> Result<SRSCardOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let repository = factory.legacy_srs_data_repository();
                if repository
                    .find_by_question(input.question_id.clone(), false)
                    .await
                    .is_some()
                {
                    return Err(format!("SrsData is exist: {}", input.question_id));
                }
                let now = chrono::Utc::now().timestamp();
                let model = repository
                    .create(NewSrsData {
                        id: Uuid::new_v4().to_string(),
                        question_id: input.question_id,
                        stability: SrsData::INITIAL_STABILITY,
                        difficulty: input.difficulty.unwrap_or(SrsData::INITIAL_DIFFICULTY),
                        next_review_at: Some(now + 24 * 3600),
                        last_reviewed_at: Some(now),
                        review_count: 1,
                        feedback_history: "[]".into(),
                        now,
                    })
                    .await;
                Ok(card(model, now, false))
            })
        })
        .await
}

/// 获取待复习的题目列表
#[tauri::command]
pub async fn get_due_questions(
    state: State<'_, AppState>,
    limit: Option<i32>,
) -> Result<Vec<SRSCardOutput>, String> {
    let now = chrono::Utc::now().timestamp();
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let mut cards: Vec<_> = factory
                    .legacy_srs_data_repository()
                    .list_active()
                    .await
                    .into_iter()
                    .filter(|s| {
                        s.next_review_at()
                            .map(|n| now >= n.timestamp())
                            .unwrap_or(true)
                    })
                    .collect();
                cards.sort_by(|a, b| a.stability().total_cmp(&b.stability()));
                cards.truncate(limit.unwrap_or(1000).max(0) as usize);
                Ok(cards.into_iter().map(|s| card(s, now, true)).collect())
            })
        })
        .await
}

/// 提交复习结果并更新 SRS 状态
#[tauri::command]
pub async fn submit_review_result(
    state: State<'_, AppState>,
    input: SubmitReviewInput,
) -> Result<ReviewOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let now = chrono::Utc::now();
                let now_timestamp = now.timestamp();
                let repository = factory.legacy_srs_data_repository();
                let mut model = repository
                    .find_by_question(input.question_id, false)
                    .await
                    .ok_or_else(|| "SRS data not found".to_string())?;
                let next_interval_days = model
                    .review(now, input.feedback)
                    .map_err(|error| error.to_string())?;
                repository
                    .update_state(SrsStateChanges {
                        id: model.question_id.to_string(),
                        stability: model.stability(),
                        difficulty: model.difficulty(),
                        next_review_at: model.next_review_at().map(|value| value.timestamp()),
                        last_reviewed_at: model.last_review_at().map(|value| value.timestamp()),
                        review_count: model.review_count().clamp(i32::MIN as i64, i32::MAX as i64)
                            as i32,
                        feedback_history: serde_json::to_string(model.feedback_history())
                            .expect("SRS feedback history is always serializable"),
                        deleted_at: None,
                        now: now_timestamp,
                    })
                    .await;
                Ok(ReviewOutput {
                    next_interval_days,
                    new_stability: model.stability(),
                    new_difficulty: model.difficulty(),
                    next_review_at: model
                        .next_review_at()
                        .expect("review always schedules the next review")
                        .timestamp(),
                })
            })
        })
        .await
}

/// 获取单个题目的 SRS 状态
#[tauri::command]
pub async fn get_question_srs_status(
    state: State<'_, AppState>,
    question_id: String,
) -> Result<Option<SRSCardOutput>, String> {
    let now = chrono::Utc::now().timestamp();
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_srs_data_repository()
                    .find_by_question(question_id, true)
                    .await
                    .map(|s| {
                        let due = s
                            .next_review_at()
                            .map(|n| n.timestamp() <= now)
                            .unwrap_or(true);
                        card(s, now, due)
                    }))
            })
        })
        .await
}

/// 将单个题目的 SRS 进度重置为新卡片状态
#[tauri::command]
pub async fn reset_srs_progress(
    state: State<'_, AppState>,
    question_id: String,
) -> Result<SRSCardOutput, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let now = chrono::Utc::now();
                let now_timestamp = now.timestamp();
                let repository = factory.legacy_srs_data_repository();
                let model = if let Some(mut model) = repository
                    .find_by_question(question_id.clone(), false)
                    .await
                {
                    model.reset(now);
                    repository
                        .update_state(SrsStateChanges {
                            id: model.question_id.to_string(),
                            stability: model.stability(),
                            difficulty: model.difficulty(),
                            next_review_at: model.next_review_at().map(|value| value.timestamp()),
                            last_reviewed_at: model.last_review_at().map(|value| value.timestamp()),
                            review_count: model.review_count() as i32,
                            feedback_history: serde_json::to_string(model.feedback_history())
                                .expect("SRS feedback history is always serializable"),
                            deleted_at: None,
                            now: now_timestamp,
                        })
                        .await
                } else {
                    repository
                        .create(NewSrsData {
                            id: Uuid::new_v4().to_string(),
                            question_id,
                            stability: SrsData::INITIAL_STABILITY,
                            difficulty: SrsData::INITIAL_DIFFICULTY,
                            next_review_at: Some(now_timestamp),
                            last_reviewed_at: Some(now_timestamp),
                            review_count: 1,
                            feedback_history: "[]".into(),
                            now: now_timestamp,
                        })
                        .await
                };
                Ok(card(model, now_timestamp, true))
            })
        })
        .await
}

// ==================== Tool/API Functions ====================

/// 获取当前待复习卡片总数
#[tauri::command]
pub async fn get_due_count(state: State<'_, AppState>) -> Result<i32, String> {
    let now = chrono::Utc::now().timestamp();
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_srs_data_repository()
                    .list_active()
                    .await
                    .into_iter()
                    .filter(|s| {
                        s.next_review_at()
                            .map(|n| now >= n.timestamp())
                            .unwrap_or(true)
                    })
                    .count() as i32)
            })
        })
        .await
}

/// 获取所有 SRS 数据的统计信息
#[tauri::command]
pub async fn get_srs_statistics(state: State<'_, AppState>) -> Result<SRSStatistics, String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                let all = factory.legacy_srs_data_repository().list_active().await;
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
                    .filter(|s| {
                        s.next_review_at()
                            .map(|n| now >= n.timestamp())
                            .unwrap_or(true)
                    })
                    .count() as i32;
                let new_cards = all.iter().filter(|s| s.review_count() == 1).count() as i32;
                let avg_stability =
                    all.iter().map(|s| s.stability() as f64).sum::<f64>() / all.len() as f64;
                let avg_difficulty =
                    all.iter().map(|s| s.difficulty() as f64).sum::<f64>() / all.len() as f64;
                let total_reviews = all.iter().map(SrsData::review_count).sum();
                Ok(SRSStatistics {
                    total,
                    due_count,
                    new_cards,
                    avg_stability: avg_stability as f32,
                    avg_difficulty: avg_difficulty as f32,
                    total_reviews,
                })
            })
        })
        .await
}

/// 获取所有卡片
#[tauri::command]
pub async fn get_all_cards(state: State<'_, AppState>) -> Result<Vec<SRSCardOutput>, String> {
    let now = chrono::Utc::now().timestamp();
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                Ok(factory
                    .legacy_srs_data_repository()
                    .list_active()
                    .await
                    .into_iter()
                    .map(|s| {
                        let due = s
                            .next_review_at()
                            .map(|n| n.timestamp() <= now)
                            .unwrap_or(true);
                        card(s, now, due)
                    })
                    .collect())
            })
        })
        .await
}

/// 根据 ID 插入或更新同步下发的 SRS 数据
#[tauri::command]
pub async fn upsert_srs_data(
    state: State<'_, AppState>,
    input: UpsertSRSDataInput,
) -> Result<(), String> {
    state
        .repository_transaction_executor
        .execute(|factory, _| {
            Box::pin(async move {
                factory
                    .legacy_srs_data_repository()
                    .upsert_synced(SyncedSrsData {
                        id: input.id,
                        version: input.version,
                        deleted_at: input.deleted_at,
                        question_id: input.question_id,
                        stability: input.stability,
                        difficulty: input.difficulty,
                        next_review_at: input.next_review_at,
                        last_reviewed_at: input.last_review_at,
                        review_count: input.review_count,
                        feedback_history: input.feedback_history,
                        now: chrono::Utc::now().timestamp(),
                    })
                    .await;
                Ok(())
            })
        })
        .await
}
