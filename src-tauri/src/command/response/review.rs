use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::application::result::srs_data::LibraryStatistics;
use crate::domain::model::SrsData as DomainSrsData;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SrsData {
    pub(crate) question_id: String,
    pub(crate) stability: f32,
    pub(crate) difficulty: f32,
    pub(crate) retrievability: f32,
    pub(crate) next_review_at: Option<String>,
    pub(crate) last_review_at: Option<String>,
    pub(crate) review_count: i64,
    pub(crate) is_due: bool,
}
impl SrsData {
    pub(crate) fn from_domain(value: DomainSrsData, at: DateTime<Utc>) -> Self {
        Self {
            question_id: value.question_id.to_string(),
            stability: value.stability(),
            difficulty: value.difficulty(),
            retrievability: value.retrievability_at(at.timestamp()),
            next_review_at: value.next_review_at().map(|date| date.to_rfc3339()),
            last_review_at: value.last_review_at().map(|date| date.to_rfc3339()),
            review_count: value.review_count(),
            is_due: value.next_review_at().is_none_or(|date| date <= at),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitReviewResponse {
    pub(crate) srs: SrsData,
    pub(crate) next_interval_days: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetReviewProgressResponse {
    pub(crate) srs: SrsData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSrsDataResponse {
    pub(crate) srs: SrsData,
}

impl GetSrsDataResponse {
    pub(crate) fn from_domain(at: DateTime<Utc>, srs: DomainSrsData) -> Self {
        Self {
            srs: SrsData::from_domain(srs, at),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSrsDataResponse {
    pub(crate) items: Vec<SrsData>,
}

impl ListSrsDataResponse {
    pub(crate) fn from_domain(at: DateTime<Utc>, items: Vec<DomainSrsData>) -> Self {
        Self {
            items: items
                .into_iter()
                .map(|srs| SrsData::from_domain(srs, at))
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStatisticsData {
    pub(crate) question_total: u64,
    pub(crate) card_total: usize,
    pub(crate) due_count: usize,
    pub(crate) new_card_count: usize,
    pub(crate) average_stability: f32,
    pub(crate) average_difficulty: f32,
    pub(crate) total_reviews: i64,
}

impl From<LibraryStatistics> for LibraryStatisticsData {
    fn from(value: LibraryStatistics) -> Self {
        Self {
            question_total: value.question_total,
            card_total: value.card_total,
            due_count: value.due_count,
            new_card_count: value.new_card_count,
            average_stability: value.average_stability,
            average_difficulty: value.average_difficulty,
            total_reviews: value.total_reviews,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLibraryStatisticsResponse {
    pub(crate) statistics: LibraryStatisticsData,
}
