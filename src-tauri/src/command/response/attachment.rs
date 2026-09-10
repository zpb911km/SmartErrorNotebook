use serde::Serialize;

use crate::domain::model::Attachment;
use crate::util::codec::encode_base64;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AttachmentData {
    pub(crate) id: String,
    pub(crate) mime_type: String,
    pub(crate) base64_data: String,
    pub(crate) sha256: String,
}

impl From<Attachment> for AttachmentData {
    fn from(value: Attachment) -> Self {
        Self {
            id: value.id.to_string(),
            mime_type: value.mime_type,
            base64_data: encode_base64(&value.data),
            sha256: value.sha256,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttachmentResponse {
    pub(crate) id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAttachmentResponse {
    pub(crate) id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAttachmentResponse {
    pub(crate) attachment: AttachmentData,
}
