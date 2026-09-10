use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreateTagCommand {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UpdateTagCommand {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeleteTagCommand {
    pub id: Uuid,
}
