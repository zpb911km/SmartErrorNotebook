pub struct NewSubject {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub now: i64,
}
pub struct SubjectChanges {
    pub id: String,
    pub name: Option<String>,
    pub color: Option<String>,
    pub now: i64,
}
pub struct SyncedSubject {
    pub id: String,
    pub version: i32,
    pub deleted_at: Option<i64>,
    pub name: String,
    pub color: Option<String>,
    pub now: i64,
}
