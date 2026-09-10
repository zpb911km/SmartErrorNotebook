use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateSubjectRequest {
    pub(crate) name: String,
    pub(crate) color: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateSubjectRequest {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) color: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteSubjectRequest {
    pub(crate) id: String,
}
