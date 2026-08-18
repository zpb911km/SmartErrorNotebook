use crate::model::Tag;

pub struct TagWithQuestion {
    pub tag: Tag,
    pub question_id: Option<String>,
}

pub struct NewErrorTag {
    pub id: String,
    pub question_id: String,
    pub name: String,
    pub color: String,
    pub now: i64,
}
pub struct SyncedErrorTag {
    pub id: String,
    pub version: i32,
    pub deleted_at: Option<i64>,
    pub question_ids: Vec<String>,
    pub name: String,
    pub color: String,
    pub now: i64,
}
