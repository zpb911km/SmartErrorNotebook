// SRS 数据相关命令 - 基于连续反馈的 SDR 模型

use crate::AppState;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use tauri::State;
use uuid::Uuid;

use crate::database::entities::prelude::SrsData;
use crate::database::entities::srs_data;
use crate::srs::{
    config, days_elapsed, predict_retrievability, review_card, update_feedback_history,
};

// ==================== Input/Output Types ====================

/// 创建/初始化 SRS 数据的输入参数
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateSRSDataInput {
    /// 错题 ID
    pub question_id: String,
    /// 初始难度 (可选，默认使用 config::INITIAL_DIFFICULTY)
    pub difficulty: Option<f32>,
}

// 用于同步的 UPSERT 输入
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
    /// 用户连续反馈值 [0, 1]，映射到 FSRS-5 四级评分：
    ///   [0.0, 0.2) → Again(1) — 遗忘路径 (post-lapse stability)
    ///   [0.2, 0.6) → Hard(2) — 成功路径，惩罚性增长
    ///   [0.6, 0.9) → Good(3) — 成功路径，标准增长
    ///   [0.9, 1.0] → Easy(4) — 成功路径，奖励性增长
    ///   边界值：0.0 = 重置为新卡片，1.0 = 永久记忆
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
    /// 下次复习间隔 (天)
    pub next_interval_days: f32,
    /// 更新后的稳定性
    pub new_stability: f32,
    /// 更新后的难度
    pub new_difficulty: f32,
    /// 建议的下次复习时间戳 (秒)
    pub next_review_at: i64,
}

/// SRS 统计数据输出
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SRSStatistics {
    /// 总卡片数
    pub total: i32,
    /// 待复习数量
    pub due_count: i32,
    /// 新卡片数量（review_count=1）
    pub new_cards: i32,
    /// 平均稳定性（天）
    pub avg_stability: f32,
    /// 平均难度
    pub avg_difficulty: f32,
    /// 总复习次数
    pub total_reviews: i64,
}

// ==================== Command Functions ====================

/// 为指定题目创建/初始化 SRS 数据
/// 这应该在使用者"添加错题"时调用，作为第一次复习
#[tauri::command]
pub async fn create_srs_data(
    state: State<'_, AppState>,
    input: CreateSRSDataInput,
) -> Result<SRSCardOutput, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();
    let id = Uuid::new_v4().to_string();

    // 验证错题是否存在（通过验证 SRS 数据是否存在）
    if !SrsData::find()
        .filter(srs_data::Column::QuestionId.eq(input.question_id.clone()))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .is_none()
    {
        return Err(format!("SrsData is exist: {}", input.question_id));
    }

    let new_srs_data = srs_data::ActiveModel {
        id: Set(id.clone()),
        question_id: Set(input.question_id.clone()),
        stability: Set(config::INITIAL_STABILITY),
        difficulty: Set(input.difficulty.unwrap_or(config::INITIAL_DIFFICULTY)),
        next_review_at: Set(Some(now + 24 * 3600)), // 默认 1 天后
        lastreviewed_at: Set(Some(now)),
        review_count: Set(1),
        feedback_history: Set("[]".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        version: Set(0),
        sync_status: Set("pending".to_string()),
        sync_hash: Set(None),
        deleted_at: Set(None),
    };

    let srs_model = new_srs_data.insert(db).await.map_err(|e| e.to_string())?;

    // 转为统一输出格式（SRSCardOutput 使用 last_review_at 字段名）
    Ok(SRSCardOutput {
        id: srs_model.id,
        question_id: srs_model.question_id,
        stability: srs_model.stability,
        difficulty: srs_model.difficulty,
        recall_rate: predict_retrievability(
            srs_model.stability,
            days_elapsed(srs_model.lastreviewed_at, now),
        ),
        next_review_at: srs_model.next_review_at,
        last_review_at: srs_model.lastreviewed_at,
        review_count: srs_model.review_count,
        is_due: false,
    })
}

/// 获取待复习的题目列表
#[tauri::command]
pub async fn get_due_questions(
    state: State<'_, AppState>,
    limit: Option<i32>,
) -> Result<Vec<SRSCardOutput>, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();
    let limit = limit.unwrap_or(1000);

    // 获取所有 SRS 数据（排除已软删除的）
    let all_srs = SrsData::find()
        .filter(srs_data::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    // 过滤出到期的题目
    let mut due_cards: Vec<srs_data::Model> = all_srs
        .into_iter()
        .filter(|srs| {
            match srs.next_review_at {
                None => true,              // 未安排复习，视为待复习
                Some(next) => now >= next, // 已过下次复习时间
            }
        })
        .collect();

    // 按稳定性升序排序 (稳定性低的优先复习)
    due_cards.sort_by(|a, b| a.stability.partial_cmp(&b.stability).unwrap());

    // 限制数量
    due_cards.truncate(limit as usize);

    // 转换为输出格式
    Ok(due_cards
        .into_iter()
        .map(|srs| SRSCardOutput {
            id: srs.id,
            question_id: srs.question_id,
            stability: srs.stability,
            difficulty: srs.difficulty,
            recall_rate: predict_retrievability(
                srs.stability,
                days_elapsed(srs.lastreviewed_at, now),
            ),
            next_review_at: srs.next_review_at,
            last_review_at: srs.lastreviewed_at,
            review_count: srs.review_count,
            is_due: true,
        })
        .collect())
}

/// 提交复习结果并更新 SRS 状态
#[tauri::command]
pub async fn submit_review_result(
    state: State<'_, AppState>,
    input: SubmitReviewInput,
) -> Result<ReviewOutput, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    // 获取当前 SRS 记录（按 question_id 查询）
    let srs_model = SrsData::find()
        .filter(srs_data::Column::QuestionId.eq(&input.question_id))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("SRS data not found".to_string())?;

    // 计算新的 SRS 状态
    let result = review_card(&srs_model, now, input.feedback)?;

    // 更新数据库
    let mut update_model = srs_model.clone().into_active_model();
    update_model.stability = Set(result.new_stability);
    update_model.difficulty = Set(result.new_difficulty);
    update_model.next_review_at = Set(Some(result.next_review_at));
    update_model.lastreviewed_at = Set(Some(now));
    update_model.review_count = Set(srs_model.review_count + 1);
    update_model.feedback_history = Set(update_feedback_history(
        &srs_model.feedback_history,
        input.feedback,
    ));
    update_model.updated_at = Set(now);
    update_model.version = Set(srs_model.version);
    update_model.sync_status = Set("pending".to_string());
    update_model.deleted_at = Set(None);
    update_model.update(db).await.map_err(|e| e.to_string())?;

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
    let db = state.db.as_ref();

    let srs_model = SrsData::find()
        .filter(srs_data::Column::QuestionId.eq(&question_id))
        .filter(srs_data::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(srs_model.map(|srs| {
        let elapsed_days = days_elapsed(srs.lastreviewed_at, chrono::Utc::now().timestamp());
        let recall_rate = predict_retrievability(srs.stability, elapsed_days);
        let is_due = srs.next_review_at.map(|next| next <= chrono::Utc::now().timestamp()).unwrap_or(true);

        SRSCardOutput {
            id: srs.id,
            question_id: srs.question_id,
            stability: srs.stability,
            difficulty: srs.difficulty,
            recall_rate: recall_rate,
            next_review_at: srs.next_review_at,
            last_review_at: srs.lastreviewed_at,
            review_count: srs.review_count,
            is_due: is_due,
        }
    }))
}

/// 重置单个题目的 SRS 进度 (当作新卡片)
#[tauri::command]
pub async fn reset_srs_progress(
    state: State<'_, AppState>,
    question_id: String,
) -> Result<SRSCardOutput, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    // 查找现有 SRS 记录（排除已软删除的）
    let srs_model = SrsData::find()
        .filter(srs_data::Column::QuestionId.eq(&question_id))
        .filter(srs_data::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    match srs_model {
        Some(model) => {
            // 更新为初始状态
            let mut update_model = model.clone().into_active_model();
            update_model.stability = Set(config::INITIAL_STABILITY);
            update_model.difficulty = Set(config::INITIAL_DIFFICULTY);
            update_model.next_review_at = Set(Some(now));
            update_model.lastreviewed_at = Set(Some(now));
            update_model.review_count = Set(1);
            update_model.feedback_history = Set("[]".to_string());
            update_model.updated_at = Set(now);
            update_model.version = Set(model.version);
            update_model.sync_status = Set("pending".to_string());

            let updated = update_model.update(db).await.map_err(|e| e.to_string())?;

            // 转为统一输出格式
            Ok(SRSCardOutput {
                id: updated.id,
                question_id: updated.question_id,
                stability: updated.stability,
                difficulty: updated.difficulty,
                recall_rate: predict_retrievability(
                    updated.stability,
                    days_elapsed(updated.lastreviewed_at, now),
                ),
                next_review_at: updated.next_review_at,
                last_review_at: updated.lastreviewed_at,
                review_count: updated.review_count,
                is_due: true,
            })
        }
        None => {
            // 如果不存在，创建新的
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
}

// ==================== Tool/API Functions ====================

/// 获取当前待复习卡片总数
#[tauri::command]
pub async fn get_due_count(state: State<'_, AppState>) -> Result<i32, String> {
    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    let all_srs = SrsData::find()
        .filter(srs_data::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let count = all_srs
        .into_iter()
        .filter(|srs| match srs.next_review_at {
            None => true,
            Some(next) => now >= next,
        })
        .count() as i32;

    Ok(count)
}

/// 获取所有 SRS 数据的统计信息
#[tauri::command]
pub async fn get_srs_statistics(state: State<'_, AppState>) -> Result<SRSStatistics, String> {
    let db = state.db.as_ref();

    let all_srs = SrsData::find()
        .filter(srs_data::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    if all_srs.is_empty() {
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
    let mut due_count = 0;
    let mut new_cards = 0;
    let mut stability_sum = 0.0;
    let mut difficulty_sum = 0.0;
    let mut total_reviews = 0;

    for srs in &all_srs {
        if srs.next_review_at.is_none() || now >= srs.next_review_at.unwrap() {
            println!("Due: {}", srs.question_id);
            due_count += 1;
        }
        if srs.review_count == 1 {
            new_cards += 1;
        }
        stability_sum += srs.stability as f64;
        difficulty_sum += srs.difficulty as f64;
        total_reviews += srs.review_count as i64;
    }
    println!("Total: {}", all_srs.len());

    Ok(SRSStatistics {
        total: all_srs.len() as i32,
        due_count,
        new_cards,
        avg_stability: (stability_sum / all_srs.len() as f64) as f32,
        avg_difficulty: (difficulty_sum / all_srs.len() as f64) as f32,
        total_reviews,
    })
}

/// 获取所有卡片的列表（可用于批量显示）
#[tauri::command]
pub async fn get_all_cards(state: State<'_, AppState>) -> Result<Vec<SRSCardOutput>, String> {
    let db = state.db.as_ref();
    let all_srs = SrsData::find()
        .filter(srs_data::Column::DeletedAt.is_null())
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(all_srs
        .into_iter()
        .map(|srs| SRSCardOutput {
            id: srs.id,
            question_id: srs.question_id,
            stability: srs.stability,
            difficulty: srs.difficulty,
            recall_rate: predict_retrievability(
                srs.stability,
                days_elapsed(srs.lastreviewed_at, chrono::Utc::now().timestamp()),
            ),
            next_review_at: srs.next_review_at,
            last_review_at: srs.lastreviewed_at,
            review_count: srs.review_count,
            is_due: srs.next_review_at.map(|next| next <= chrono::Utc::now().timestamp()).unwrap_or(true),
        })
        .collect())
}

/// UPSERT: 根据 ID 插入或更新 SRS 数据（用于同步）
#[tauri::command]
pub async fn upsert_srs_data(
    state: State<'_, AppState>,
    input: UpsertSRSDataInput,
) -> Result<(), String> {
    use crate::database::entities::srs_data as srs;

    let db = state.db.as_ref();
    let now = chrono::Utc::now().timestamp();

    // 检查记录是否存在
    let existing = srs::Entity::find_by_id(input.id.clone())
        .one(db)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

    if let Some(model) = existing {
        // 更新现有记录
        let mut active_model: srs_data::ActiveModel = model.into();
        active_model.question_id = Set(input.question_id);
        active_model.stability = Set(input.stability);
        active_model.difficulty = Set(input.difficulty);
        active_model.next_review_at = Set(input.next_review_at);
        active_model.lastreviewed_at = Set(input.lastreviewed_at);
        active_model.review_count = Set(input.review_count);
        active_model.feedback_history = Set(input.feedback_history);
        active_model.updated_at = Set(now);
        active_model.version = Set(input.version);
        active_model.sync_status = Set("synced".to_string());
        active_model.deleted_at = Set(input.deleted_at);
        active_model.update(db).await.map_err(|e| e.to_string())?;
    } else {
        // 插入新记录
        let new_srs = srs_data::ActiveModel {
            id: Set(input.id),
            question_id: Set(input.question_id),
            stability: Set(input.stability),
            difficulty: Set(input.difficulty),
            next_review_at: Set(input.next_review_at),
            lastreviewed_at: Set(input.lastreviewed_at),
            review_count: Set(input.review_count),
            feedback_history: Set(input.feedback_history),
            created_at: Set(now),
            updated_at: Set(now),
            version: Set(input.version),
            sync_status: Set("synced".to_string()),
            sync_hash: Set(None),
            deleted_at: Set(None),
        };

        let _ = new_srs.insert(db).await;
    }

    Ok(())
}
