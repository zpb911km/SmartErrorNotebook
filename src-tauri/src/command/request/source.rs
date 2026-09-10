use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateSourceRequest {
    pub(crate) subject_id: Option<String>,
    pub(crate) book: Option<String>,
    pub(crate) chapter: Option<String>,
    pub(crate) knowledge: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateSourceRequest {
    pub(crate) id: String,
    #[serde(deserialize_with = "super::deserialize_required_nullable")]
    pub(crate) subject_id: Option<String>,
    #[serde(deserialize_with = "super::deserialize_required_nullable")]
    pub(crate) book: Option<String>,
    #[serde(deserialize_with = "super::deserialize_required_nullable")]
    pub(crate) chapter: Option<String>,
    #[serde(deserialize_with = "super::deserialize_required_nullable")]
    pub(crate) knowledge: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteSourceRequest {
    pub(crate) id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteSourcesRequest {
    pub(crate) ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetSourceRequest {
    pub(crate) id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListSourcesRequest {
    pub(crate) subject_id: Option<String>,
}
