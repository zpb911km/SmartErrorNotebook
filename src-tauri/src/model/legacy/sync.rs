use super::DomainError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    Pending,
    Synced,
    Conflict,
}

impl SyncStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Synced => "synced",
            Self::Conflict => "conflict",
        }
    }
}

impl TryFrom<String> for SyncStatus {
    type Error = DomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "synced" => Ok(Self::Synced),
            "conflict" => Ok(Self::Conflict),
            _ => Err(DomainError::UnknownSyncStatus(value)),
        }
    }
}

impl From<SyncStatus> for String {
    fn from(value: SyncStatus) -> Self {
        match value {
            SyncStatus::Pending => "pending".to_owned(),
            SyncStatus::Synced => "synced".to_owned(),
            SyncStatus::Conflict => "conflict".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SyncStatus;

    #[test]
    fn accepts_supported_statuses_case_insensitively() {
        assert_eq!(
            SyncStatus::try_from("pending".to_owned()).unwrap(),
            SyncStatus::Pending
        );
        assert_eq!(
            SyncStatus::try_from("SYNCED".to_owned()).unwrap(),
            SyncStatus::Synced
        );
        assert_eq!(
            SyncStatus::try_from("Conflict".to_owned()).unwrap(),
            SyncStatus::Conflict
        );
    }

    #[test]
    fn rejects_unknown_statuses() {
        let error = SyncStatus::try_from("invalid".to_owned()).unwrap_err();
        assert_eq!(error.to_string(), "Unknown sync status: invalid");
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
