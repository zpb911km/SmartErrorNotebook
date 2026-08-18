use super::super::database::entity::{
    attachment, question, question_attachment_cross_ref, question_tag_cross_ref, source, srs_data,
    tag,
};
use super::{timestamp, uuid};
use crate::model::Question;
use crate::repository::legacy;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
};

pub struct SeaOrmQuestionRepository<'c, C: ConnectionTrait> {
    connection: &'c C,
}
impl<'c, C: ConnectionTrait> SeaOrmQuestionRepository<'c, C> {
    pub fn new(connection: &'c C) -> Self {
        Self {
            connection: connection,
        }
    }

    async fn source_for_subject(
        &self,
        subject_id: String,
        requested_source_id: Option<String>,
        now: chrono::DateTime<chrono::Utc>,
        sync_status: &str,
    ) -> uuid::Uuid {
        let subject_id = uuid(&subject_id, "subject id");
        let requested = match requested_source_id.filter(|id| !id.is_empty()) {
            Some(id) => Some(
                source::Entity::find_by_id(uuid(&id, "source id"))
                    .one(self.connection)
                    .await
                    .expect("failed to query question source")
                    .expect("Source not found"),
            ),
            None => None,
        };

        if requested
            .as_ref()
            .is_some_and(|source| source.subject_id == Some(subject_id))
        {
            return requested.unwrap().id;
        }

        let (book, chapter, knowledge) = requested
            .as_ref()
            .map(|source| {
                (
                    source.book.clone(),
                    source.chapter.clone(),
                    source.knowledge.clone(),
                )
            })
            .unwrap_or_default();
        if let Some(existing) = source::Entity::find()
            .filter(source::Column::SubjectId.eq(subject_id))
            .filter(source::Column::DeletedAt.is_null())
            .all(self.connection)
            .await
            .expect("failed to find source for subject")
            .into_iter()
            .find(|source| {
                source.book == book && source.chapter == chapter && source.knowledge == knowledge
            })
        {
            return existing.id;
        }

        let id = uuid::Uuid::new_v4();
        source::ActiveModel {
            id: Set(id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            sync_status: Set(sync_status.to_owned()),
            sync_version: Set(0),
            subject_id: Set(Some(subject_id)),
            book: Set(book),
            chapter: Set(chapter),
            knowledge: Set(knowledge),
        }
        .insert(self.connection)
        .await
        .expect("failed to create source for subject");
        id
    }

    async fn unlink_dependents(&self, id: uuid::Uuid, now: chrono::DateTime<chrono::Utc>) {
        for link in question_attachment_cross_ref::Entity::find()
            .filter(question_attachment_cross_ref::Column::QuestionId.eq(id))
            .all(self.connection)
            .await
            .expect("failed to query question attachment links")
        {
            question_attachment_cross_ref::Entity::delete_by_id((id, link.attachment_id))
                .exec(self.connection)
                .await
                .expect("failed to unlink question attachment");
            let remaining = question_attachment_cross_ref::Entity::find()
                .filter(question_attachment_cross_ref::Column::AttachmentId.eq(link.attachment_id))
                .count(self.connection)
                .await
                .expect("failed to count attachment links");
            if let Some(model) = attachment::Entity::find_by_id(link.attachment_id)
                .one(self.connection)
                .await
                .expect("failed to query linked attachment")
            {
                let mut active: attachment::ActiveModel = model.into();
                active.updated_at = Set(now);
                active.sync_status = Set("PENDING".into());
                if remaining == 0 {
                    active.deleted_at = Set(Some(now));
                }
                active
                    .update(self.connection)
                    .await
                    .expect("failed to update unlinked attachment");
            }
        }
        for link in question_tag_cross_ref::Entity::find()
            .filter(question_tag_cross_ref::Column::QuestionId.eq(id))
            .all(self.connection)
            .await
            .expect("failed to query question tag links")
        {
            question_tag_cross_ref::Entity::delete_by_id((id, link.tag_id))
                .exec(self.connection)
                .await
                .expect("failed to unlink question tag");
            let remaining = question_tag_cross_ref::Entity::find()
                .filter(question_tag_cross_ref::Column::TagId.eq(link.tag_id))
                .count(self.connection)
                .await
                .expect("failed to count tag links");
            if let Some(model) = tag::Entity::find_by_id(link.tag_id)
                .one(self.connection)
                .await
                .expect("failed to query linked tag")
            {
                let mut active: tag::ActiveModel = model.into();
                active.updated_at = Set(now);
                active.sync_status = Set("PENDING".into());
                if remaining == 0 {
                    active.deleted_at = Set(Some(now));
                }
                active
                    .update(self.connection)
                    .await
                    .expect("failed to update unlinked tag");
            }
        }
        if let Some(model) = srs_data::Entity::find_by_id(id)
            .one(self.connection)
            .await
            .expect("failed to query srs data")
        {
            let mut active: srs_data::ActiveModel = model.into();
            active.deleted_at = Set(Some(now));
            active.updated_at = Set(now);
            active.sync_status = Set("PENDING".into());
            active
                .update(self.connection)
                .await
                .expect("failed to soft-delete srs data");
        }
    }
}

fn normalized_type(value: &str) -> Option<String> {
    Some(
        match value {
            "单选题" | "SINGLE_SELECT" | "SigleChoice" | "SingleChoice" => "SINGLE_SELECT",
            "多选题" | "MULTIPLE_SELECT" | "MultipleChoice" => "MULTIPLE_SELECT",
            "判断题" | "TRUE_FALSE" | "TrueFalse" => "TRUE_FALSE",
            "填空题" | "FILL_IN_THE_BLANK" | "FillInTheBlank" => "FILL_IN_THE_BLANK",
            "简答题" | "SHORT_ANSWER" | "ShortAnswer" => "SHORT_ANSWER",
            "计算题" | "CALCULATION" | "Calculation" => "CALCULATION",
            "论述题" | "ESSAY" | "Essay" => "ESSAY",
            _ => return None,
        }
        .to_owned(),
    )
}

fn optional_uuid(value: Option<String>, field: &str) -> Option<uuid::Uuid> {
    value
        .filter(|value| !value.is_empty())
        .map(|value| uuid(&value, field))
}

#[async_trait::async_trait]
impl<'c, C: ConnectionTrait> legacy::ErrorQuestionRepository for SeaOrmQuestionRepository<'c, C> {
    async fn list_active(
        &self,
        input: legacy::repository_model::error_question::QuestionQuery,
    ) -> Vec<Question> {
        let mut query = question::Entity::find().filter(question::Column::DeletedAt.is_null());
        if let Some(id) = input.subject_id {
            query = query
                .join(
                    sea_orm::JoinType::InnerJoin,
                    question::Relation::Source.def(),
                )
                .filter(source::Column::SubjectId.eq(uuid(&id, "subject id")));
        }
        if let Some(search) = input.search {
            let pattern = format!("%{search}%");
            query = query.filter(
                question::Column::Stem
                    .like(&pattern)
                    .or(question::Column::Explanation.like(&pattern))
                    .or(question::Column::Note.like(&pattern)),
            );
        }
        query = query.order_by_desc(question::Column::UpdatedAt);
        if let Some(limit) = input.limit {
            query = query.limit(limit);
        }
        if let Some(offset) = input.offset {
            query = query.offset(offset);
        }
        query
            .all(self.connection)
            .await
            .expect("failed to list questions")
            .into_iter()
            .map(Into::into)
            .collect()
    }

    async fn find_by_id(&self, id: String) -> Question {
        question::Entity::find_by_id(uuid(&id, "question id"))
            .one(self.connection)
            .await
            .expect("failed to query question")
            .expect("Question not found")
            .into()
    }

    async fn create(
        &self,
        input: legacy::repository_model::error_question::NewQuestion,
    ) -> Question {
        let now = timestamp(input.now, "question timestamp");
        let source_id = self
            .source_for_subject(input.subject_id, input.source_id, now, "PENDING")
            .await;
        question::ActiveModel {
            id: Set(uuid(&input.id, "question id")),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            sync_status: Set("PENDING".into()),
            sync_version: Set(0),
            question_type: Set(normalized_type(&input.type_)),
            source_id: Set(Some(source_id)),
            stem: Set(input.prompt),
            correct_answer: Set(input.answer.unwrap_or_default()),
            explanation: Set(input.analysis),
            note: Set(input.error_note),
        }
        .insert(self.connection)
        .await
        .expect("failed to create question")
        .into()
    }

    async fn update(
        &self,
        input: legacy::repository_model::error_question::QuestionChanges,
    ) -> Question {
        let model = question::Entity::find_by_id(uuid(&input.id, "question id"))
            .one(self.connection)
            .await
            .expect("failed to query question")
            .expect("Question not found");
        let requested_source_id = input
            .source_id
            .clone()
            .filter(|id| !id.is_empty())
            .or_else(|| model.source_id.map(|id| id.to_string()));
        let mut active: question::ActiveModel = model.into();
        if let Some(value) = input.subject_id {
            active.source_id = Set(Some(
                self.source_for_subject(
                    value,
                    requested_source_id,
                    timestamp(input.now, "question timestamp"),
                    "PENDING",
                )
                .await,
            ));
        } else if let Some(value) = input.source_id {
            active.source_id = Set(optional_uuid(Some(value), "source id"));
        }
        if let Some(value) = input.prompt {
            active.stem = Set(value);
        }
        if let Some(value) = input.type_ {
            active.question_type = Set(normalized_type(&value));
        }
        if let Some(value) = input.answer {
            active.correct_answer = Set(value);
        }
        if let Some(value) = input.analysis {
            active.explanation = Set(Some(value));
        }
        if let Some(value) = input.error_note {
            active.note = Set(Some(value));
        }
        active.updated_at = Set(timestamp(input.now, "question timestamp"));
        active.sync_status = Set("PENDING".into());
        active
            .update(self.connection)
            .await
            .expect("failed to update question")
            .into()
    }

    async fn soft_delete_with_srs(&self, id: String, now: i64) {
        let id = uuid(&id, "question id");
        let now = timestamp(now, "question timestamp");
        self.unlink_dependents(id, now).await;
        let model = question::Entity::find_by_id(id)
            .one(self.connection)
            .await
            .expect("failed to query question")
            .expect("Question not found");
        let mut active: question::ActiveModel = model.into();
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);
        active.sync_status = Set("PENDING".into());
        active
            .update(self.connection)
            .await
            .expect("failed to soft-delete question");
    }

    async fn count_active(&self) -> u64 {
        question::Entity::find()
            .filter(question::Column::DeletedAt.is_null())
            .count(self.connection)
            .await
            .expect("failed to count questions")
    }

    async fn upsert_synced(&self, input: legacy::repository_model::error_question::SyncedQuestion) {
        let id = uuid(&input.id, "question id");
        let now = timestamp(input.now, "question timestamp");
        let deleted_at = input
            .deleted_at
            .map(|value| timestamp(value, "question deleted_at"));
        let existing = question::Entity::find_by_id(id)
            .one(self.connection)
            .await
            .expect("failed to query question");
        let source_id = if deleted_at.is_some() {
            existing.as_ref().and_then(|question| question.source_id)
        } else {
            Some(
                self.source_for_subject(input.subject_id, input.source_id, now, "SYNCED")
                    .await,
            )
        };
        let mut active = question::ActiveModel {
            id: Set(id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(deleted_at),
            sync_status: Set("SYNCED".into()),
            sync_version: Set(i64::from(input.version)),
            question_type: Set(normalized_type(&input.type_)),
            source_id: Set(source_id),
            stem: Set(input.prompt),
            correct_answer: Set(input.answer.unwrap_or_default()),
            explanation: Set(input.analysis),
            note: Set(input.error_note),
        };
        if existing.is_some() {
            active.id = sea_orm::ActiveValue::Unchanged(id);
            active
                .update(self.connection)
                .await
                .expect("failed to update synced question");
        } else {
            active
                .insert(self.connection)
                .await
                .expect("failed to insert synced question");
        }
        if deleted_at.is_some() {
            self.unlink_dependents(id, now).await;
        }
    }
}
