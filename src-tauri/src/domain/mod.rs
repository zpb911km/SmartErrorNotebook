use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("Unknown sync status: {0}")]
    UnknownSyncStatus(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    Pending,
    Synced,
    Other(String),
}

impl SyncStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Synced => "synced",
            Self::Other(value) => value,
        }
    }
}

impl TryFrom<String> for SyncStatus {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "pending" => Ok(Self::Pending),
            "synced" => Ok(Self::Synced),
            _ => Ok(Self::Other(value)),
        }
    }
}

impl From<SyncStatus> for String {
    fn from(value: SyncStatus) -> Self {
        match value {
            SyncStatus::Pending => "pending".to_owned(),
            SyncStatus::Synced => "synced".to_owned(),
            SyncStatus::Other(value) => value,
        }
    }
}

impl Serialize for SyncStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SyncStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        SyncStatus::try_from(value).map_err(serde::de::Error::custom)
    }
}

macro_rules! open_string_enum {
    (
        $name:ident {
            $($variant:ident => $value:literal),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name {
            $($variant,)+
            Other(String),
        }

        impl $name {
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $value,)+
                    Self::Other(value) => value,
                }
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                match value.as_str() {
                    $($value => Self::$variant,)+
                    _ => Self::Other(value),
                }
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                match value {
                    $($name::$variant => $value.to_owned(),)+
                    $name::Other(value) => value,
                }
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                String::deserialize(deserializer).map(Self::from)
            }
        }
    };
}

open_string_enum!(QuestionType {
    MultipleChoice => "多选题",
    ShortAnswer => "简答题",
    TrueFalse => "判断题",
    SingleChoice => "单选题",
    FillInTheBlank => "填空题",
    Essay => "论述题",
    Calculation => "计算题",
});

open_string_enum!(AttachmentType {
    Original => "original",
    Answer => "answer",
});

open_string_enum!(FileType {
    Image => "img",
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityMetadata {
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
    pub version: i32,
    pub sync_status: SyncStatus,
    pub sync_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    #[serde(flatten)]
    pub metadata: EntityMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorQuestion {
    pub id: String,
    #[serde(alias = "userid")]
    pub user_id: String,
    #[serde(alias = "subjectid")]
    pub subject_id: String,
    #[serde(alias = "sourceid")]
    pub source_id: Option<String>,
    pub prompt: String,
    #[serde(rename = "type", alias = "type_")]
    pub question_type: QuestionType,
    pub answer: Option<String>,
    pub analysis: Option<String>,
    pub error_note: Option<String>,
    #[serde(flatten)]
    pub metadata: EntityMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorTag {
    pub id: String,
    pub question_id: String,
    pub name: String,
    pub color: String,
    #[serde(flatten)]
    pub metadata: EntityMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub question_id: Option<String>,
    pub subject_id: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<String>,
    pub knowledge: Option<String>,
    #[serde(flatten)]
    pub metadata: EntityMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub question_id: String,
    #[serde(rename = "type", alias = "type_")]
    pub attachment_type: AttachmentType,
    pub file_type: FileType,
    pub data: Vec<u8>,
    pub hash: String,
    #[serde(flatten)]
    pub metadata: EntityMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SrsData {
    pub id: String,
    pub question_id: String,
    pub stability: f32,
    pub difficulty: f32,
    pub next_review_at: Option<i64>,
    #[serde(alias = "lastreviewed_at")]
    pub last_review_at: Option<i64>,
    pub review_count: i32,
    pub feedback_history: String,
    #[serde(flatten)]
    pub metadata: EntityMetadata,
}

#[cfg(test)]
mod tests {
    use super::{AttachmentType, QuestionType, SyncStatus};

    #[test]
    fn open_enums_preserve_known_and_unknown_values() {
        let known: QuestionType = serde_json::from_str("\"简答题\"").unwrap();
        assert_eq!(known, QuestionType::ShortAnswer);
        assert_eq!(serde_json::to_string(&known).unwrap(), "\"简答题\"");

        let unknown: AttachmentType = serde_json::from_str("\"audio\"").unwrap();
        assert_eq!(unknown, AttachmentType::Other("audio".to_owned()));
        assert_eq!(serde_json::to_string(&unknown).unwrap(), "\"audio\"");
    }

    #[test]
    fn sync_status_preserves_unknown_values() {
        assert_eq!(
            SyncStatus::try_from("pending".to_owned()).unwrap(),
            SyncStatus::Pending
        );
        assert_eq!(
            SyncStatus::try_from("conflict".to_owned()).unwrap(),
            SyncStatus::Other("conflict".to_owned())
        );
        assert_eq!(
            serde_json::to_string(&SyncStatus::Other("legacy".to_owned())).unwrap(),
            "\"legacy\""
        );
    }
}
