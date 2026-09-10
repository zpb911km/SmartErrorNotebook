use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateTagRequest {
    pub(crate) name: String,
    pub(crate) color: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateTagRequest {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) color: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteTagRequest {
    pub(crate) id: String,
}
