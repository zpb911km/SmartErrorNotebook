use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateQuestionRequest {
    pub(crate) source_id: Option<String>,
    pub(crate) question_type: Option<String>,
    pub(crate) stem: String,
    pub(crate) correct_answer: String,
    pub(crate) explanation: Option<String>,
    pub(crate) note: Option<String>,
    #[serde(default)]
    pub(crate) tag_ids: Vec<String>,
    #[serde(default)]
    pub(crate) attachment_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateQuestionRequest {
    pub(crate) id: String,
    #[serde(deserialize_with = "super::deserialize_required_nullable")]
    pub(crate) source_id: Option<String>,
    #[serde(deserialize_with = "super::deserialize_required_nullable")]
    pub(crate) question_type: Option<String>,
    pub(crate) stem: String,
    pub(crate) correct_answer: String,
    #[serde(deserialize_with = "super::deserialize_required_nullable")]
    pub(crate) explanation: Option<String>,
    #[serde(deserialize_with = "super::deserialize_required_nullable")]
    pub(crate) note: Option<String>,
    pub(crate) tag_ids: Vec<String>,
    pub(crate) attachment_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteQuestionRequest {
    pub(crate) id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetQuestionRequest {
    pub(crate) id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListQuestionsRequest {
    #[serde(default)]
    pub(crate) filter: QuestionFilterRequest,
    pub(crate) sort: Option<Vec<String>>,
    pub(crate) offset: Option<usize>,
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuestionFilterRequest {
    pub(crate) search: Option<String>,
    pub(crate) book: Option<String>,
    pub(crate) chapter: Option<String>,
    pub(crate) knowledge: Option<String>,
    pub(crate) tag_ids: Option<Vec<String>>,
    pub(crate) updated_since: Option<String>,
    pub(crate) review_state: Option<String>,
}
