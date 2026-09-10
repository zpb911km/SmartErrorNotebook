use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubmitReviewRequest {
    pub(crate) question_id: String,
    pub(crate) feedback: f32,
    pub(crate) reviewed_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResetReviewProgressRequest {
    pub(crate) question_id: String,
    pub(crate) reset_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetSrsDataRequest {
    pub(crate) question_id: String,
    pub(crate) at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListSrsDataRequest {
    pub(crate) at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetLibraryStatisticsRequest {
    pub(crate) at: String,
}
