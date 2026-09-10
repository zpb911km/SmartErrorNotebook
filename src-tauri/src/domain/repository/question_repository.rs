use std::collections::HashSet;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::model::Question;

use super::error::{
    RepositoryCountError, RepositoryDeleteError, RepositoryFindError, RepositorySaveError,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct QuestionFilter {
    pub source_ids: HashSet<Uuid>,
    pub search: Option<String>,
    pub subject_id: Option<Uuid>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
    pub tag_ids: HashSet<Uuid>,
    pub updated_since: Option<DateTime<Utc>>,
    pub review_state: Option<ReviewState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReviewState {
    Due(DateTime<Utc>),
    NotDue(DateTime<Utc>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QuestionSort {
    IdAsc,
    IdDesc,
    UpdatedAtAsc,
    UpdatedAtDesc,
    MasteryAsc(DateTime<Utc>),
    MasteryDesc(DateTime<Utc>),
}

#[async_trait::async_trait]
/// Collection results returned by `find_*` methods have no ordering guarantee.
pub(crate) trait QuestionRepository: Send {
    async fn save(&self, question: &Question) -> Result<(), RepositorySaveError>;
    async fn delete_by_id(&self, question_id: &Uuid) -> Result<(), RepositoryDeleteError>;
    async fn find_all(&self, include_deleted: bool) -> Result<Vec<Question>, RepositoryFindError>;
    async fn find_by_id(
        &self,
        id: &Uuid,
        include_deleted: bool,
    ) -> Result<Option<Question>, RepositoryFindError>;
    async fn find_filtered(
        &self,
        filter: QuestionFilter,
        sort: Vec<QuestionSort>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<Question>, RepositoryFindError>;
    async fn count_all(&self, include_deleted: bool) -> Result<u64, RepositoryCountError>;
    async fn count_filtered(&self, filter: QuestionFilter) -> Result<usize, RepositoryFindError>;
}
