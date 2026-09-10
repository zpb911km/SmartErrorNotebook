use tauri::State;

use crate::application::{
    self,
    command::{ResetSrsDataCommand, UpdateSrsDataCommand},
    query::GetSrsDataQuery,
};
use crate::util::parsing::{parse_datetime, parse_uuid};
use crate::AppState;

use super::error::ApiError;
use super::request::review::{
    GetLibraryStatisticsRequest, GetSrsDataRequest, ListSrsDataRequest, ResetReviewProgressRequest,
    SubmitReviewRequest,
};
use super::response::review::{
    GetLibraryStatisticsResponse, GetSrsDataResponse, LibraryStatisticsData, ListSrsDataResponse,
    ResetReviewProgressResponse, SrsData, SubmitReviewResponse,
};

#[tauri::command]
pub async fn submit_review(
    state: State<'_, AppState>,
    request: SubmitReviewRequest,
) -> Result<SubmitReviewResponse, ApiError> {
    let question_id = parse_uuid(&request.question_id, "questionId")?;
    let reviewed_at = parse_datetime(&request.reviewed_at, "reviewedAt")?;
    let result = application::srs_data::update_srs_data(
        &state.repository_transaction_executor,
        UpdateSrsDataCommand {
            question_id,
            review_feedback: request.feedback,
            reviewed_at,
        },
    )
    .await?;
    Ok(SubmitReviewResponse {
        srs: SrsData::from_domain(result.srs, reviewed_at),
        next_interval_days: result.next_interval_days,
    })
}

#[tauri::command]
pub async fn reset_review_progress(
    state: State<'_, AppState>,
    request: ResetReviewProgressRequest,
) -> Result<ResetReviewProgressResponse, ApiError> {
    let question_id = parse_uuid(&request.question_id, "questionId")?;
    let reset_at = parse_datetime(&request.reset_at, "resetAt")?;
    let srs = application::srs_data::reset_srs_data(
        &state.repository_transaction_executor,
        ResetSrsDataCommand {
            question_id,
            reset_at,
        },
    )
    .await?;
    Ok(ResetReviewProgressResponse {
        srs: SrsData::from_domain(srs, reset_at),
    })
}

#[tauri::command]
pub async fn get_srs_data(
    state: State<'_, AppState>,
    request: GetSrsDataRequest,
) -> Result<GetSrsDataResponse, ApiError> {
    Ok(GetSrsDataResponse::from_domain(
        parse_datetime(&request.at, "at")?,
        application::srs_data::get_srs_data(
            &state.repository_transaction_executor,
            GetSrsDataQuery::ByQuestionId(parse_uuid(&request.question_id, "questionId")?),
        )
        .await?,
    ))
}

#[tauri::command]
pub async fn list_srs_data(
    state: State<'_, AppState>,
    request: ListSrsDataRequest,
) -> Result<ListSrsDataResponse, ApiError> {
    Ok(ListSrsDataResponse::from_domain(
        parse_datetime(&request.at, "at")?,
        application::srs_data::list_srs_data(&state.repository_transaction_executor).await?,
    ))
}

#[tauri::command]
pub async fn get_library_statistics(
    state: State<'_, AppState>,
    request: GetLibraryStatisticsRequest,
) -> Result<GetLibraryStatisticsResponse, ApiError> {
    let at = parse_datetime(&request.at, "at")?;
    let value =
        application::srs_data::get_library_statistics(&state.repository_transaction_executor, at)
            .await?;
    Ok(GetLibraryStatisticsResponse {
        statistics: LibraryStatisticsData::from(value),
    })
}
