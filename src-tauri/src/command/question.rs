use std::collections::HashSet;

use chrono::Utc;
use tauri::State;

use crate::application::{
    self,
    command::{CreateQuestionCommand, DeleteQuestionCommand, UpdateQuestionCommand},
    query::{GetQuestionQuery, ListQuestionsQuery, QuestionFilter, QuestionSort, ReviewState},
};
use crate::domain::model::QuestionType;
use crate::util::parsing::{parse_datetime, parse_uuid};
use crate::AppState;

use super::error::ApiError;
use super::request::question::{
    CreateQuestionRequest, DeleteQuestionRequest, GetQuestionRequest, ListQuestionsRequest,
    UpdateQuestionRequest,
};
use super::response::question::{
    CreateQuestionResponse, DeleteQuestionResponse, GetQuestionResponse, ListQuestionsResponse,
    UpdateQuestionResponse,
};

#[tauri::command]
pub async fn create_question(
    state: State<'_, AppState>,
    request: CreateQuestionRequest,
) -> Result<CreateQuestionResponse, ApiError> {
    let command = CreateQuestionCommand {
        source_id: request
            .source_id
            .map(|value| parse_uuid(&value, "sourceId"))
            .transpose()?,
        question_type: request
            .question_type
            .map(|value| {
                QuestionType::from_str(&value)
                    .map_err(|_| ApiError::invalid("questionType is not supported"))
            })
            .transpose()?,
        stem: request.stem,
        correct_answer: request.correct_answer,
        explanation: request.explanation,
        note: request.note,
        tag_ids: request
            .tag_ids
            .into_iter()
            .map(|value| parse_uuid(&value, "tagId"))
            .collect::<Result<Vec<_>, _>>()?,
        attachment_ids: request
            .attachment_ids
            .into_iter()
            .map(|value| parse_uuid(&value, "attachmentId"))
            .collect::<Result<Vec<_>, _>>()?,
    };
    let question =
        application::question::create_question(&state.repository_transaction_executor, command)
            .await?;
    Ok(CreateQuestionResponse {
        question: question.into(),
    })
}

#[tauri::command]
pub async fn update_question(
    state: State<'_, AppState>,
    request: UpdateQuestionRequest,
) -> Result<UpdateQuestionResponse, ApiError> {
    let id = parse_uuid(&request.id, "id")?;
    let command = UpdateQuestionCommand {
        id,
        source_id: request
            .source_id
            .map(|value| parse_uuid(&value, "sourceId"))
            .transpose()?,
        question_type: request
            .question_type
            .map(|value| {
                QuestionType::from_str(&value)
                    .map_err(|_| ApiError::invalid("questionType is not supported"))
            })
            .transpose()?,
        stem: request.stem,
        correct_answer: request.correct_answer,
        explanation: request.explanation,
        note: request.note,
        tag_ids: request
            .tag_ids
            .into_iter()
            .map(|value| parse_uuid(&value, "tagId"))
            .collect::<Result<Vec<_>, _>>()?,
        attachment_ids: request
            .attachment_ids
            .into_iter()
            .map(|value| parse_uuid(&value, "attachmentId"))
            .collect::<Result<Vec<_>, _>>()?,
    };
    let question =
        application::question::update_question(&state.repository_transaction_executor, command)
            .await?;
    Ok(UpdateQuestionResponse {
        question: question.into(),
    })
}

#[tauri::command]
pub async fn delete_question(
    state: State<'_, AppState>,
    request: DeleteQuestionRequest,
) -> Result<DeleteQuestionResponse, ApiError> {
    let id = parse_uuid(&request.id, "id")?;
    application::question::delete_question(
        &state.repository_transaction_executor,
        DeleteQuestionCommand { id },
    )
    .await?;
    Ok(DeleteQuestionResponse { id: id.to_string() })
}

#[tauri::command]
pub async fn get_question(
    state: State<'_, AppState>,
    request: GetQuestionRequest,
) -> Result<GetQuestionResponse, ApiError> {
    Ok(GetQuestionResponse {
        question: application::question::get_question(
            &state.repository_transaction_executor,
            GetQuestionQuery::ById(parse_uuid(&request.id, "id")?),
        )
        .await?
        .into(),
    })
}

#[tauri::command]
pub async fn list_questions(
    state: State<'_, AppState>,
    request: ListQuestionsRequest,
) -> Result<ListQuestionsResponse, ApiError> {
    let at = Utc::now();
    let filter = QuestionFilter {
        source_ids: HashSet::new(),
        review_state: match request.filter.review_state.as_deref() {
            None => None,
            Some("DUE") => Some(ReviewState::Due(at)),
            Some("NOT_DUE") => Some(ReviewState::NotDue(at)),
            Some(_) => return Err(ApiError::invalid("reviewState is not supported")),
        },
        tag_ids: request
            .filter
            .tag_ids
            .unwrap_or_default()
            .into_iter()
            .map(|value| parse_uuid(&value, "tagId"))
            .collect::<Result<HashSet<_>, _>>()?,
        updated_since: request
            .filter
            .updated_since
            .map(|value| parse_datetime(&value, "updatedSince"))
            .transpose()?,
        search: request.filter.search,
        subject_id: None,
        book: request.filter.book,
        chapter: request.filter.chapter,
        knowledge: request.filter.knowledge,
    };
    let sort = request
        .sort
        .unwrap_or_default()
        .into_iter()
        .map(|value| match value.as_str() {
            "UPDATED_AT_ASC" => Ok(QuestionSort::UpdatedAtAsc),
            "UPDATED_AT_DESC" => Ok(QuestionSort::UpdatedAtDesc),
            "MASTERY_ASC" => Ok(QuestionSort::MasteryAsc(at)),
            "MASTERY_DESC" => Ok(QuestionSort::MasteryDesc(at)),
            "ID_ASC" => Ok(QuestionSort::IdAsc),
            "ID_DESC" => Ok(QuestionSort::IdDesc),
            _ => Err(ApiError::invalid("sort contains an unsupported value")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let page = application::question::list_questions_with_total(
        &state.repository_transaction_executor,
        ListQuestionsQuery::Filtered {
            filter: Box::new(filter),
            sort,
            offset: request.offset,
            limit: request.limit,
        },
    )
    .await?;
    Ok(ListQuestionsResponse {
        items: page.items.into_iter().map(Into::into).collect(),
        total: page.total,
    })
}
