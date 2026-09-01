use crate::{
    domain::model::Attachment,
    util::{codec, legacy::codec as legacy_codec},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AttachmentInterface {
    pub id: String,
    pub question_id: String,
    #[serde(rename = "type")]
    pub attachment_type: String,
    #[serde(rename = "type_")]
    pub legacy_attachment_type: String,
    pub file_type: String,
    pub base64_data: String,
    pub hash: String,
}
impl AttachmentInterface {
    pub fn from_model(model: Attachment, question_id: String) -> Self {
        Self {
            id: model.id.to_string(),
            question_id,
            attachment_type: "original".into(),
            legacy_attachment_type: "original".into(),
            file_type: legacy_codec::legacy_file_type(&model.mime_type),
            base64_data: codec::encode_base64(&model.data),
            hash: model.sha256,
        }
    }
}
