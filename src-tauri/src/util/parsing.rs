use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{field} must be {expected}")]
pub(crate) struct ValueParseError {
    field: String,
    expected: &'static str,
}

pub(crate) fn parse_uuid(value: &str, field: &str) -> Result<Uuid, ValueParseError> {
    Uuid::parse_str(value).map_err(|_| ValueParseError {
        field: field.to_owned(),
        expected: "a UUID",
    })
}

pub(crate) fn parse_datetime(value: &str, field: &str) -> Result<DateTime<Utc>, ValueParseError> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .map_err(|_| ValueParseError {
            field: field.to_owned(),
            expected: "RFC 3339",
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ids_and_reports_the_field() {
        let id = Uuid::new_v4();
        assert_eq!(parse_uuid(&id.to_string(), "questionId").unwrap(), id);
        assert_eq!(
            parse_uuid("", "questionId").unwrap_err().to_string(),
            "questionId must be a UUID"
        );
    }

    #[test]
    fn parses_rfc3339_times_as_utc_and_reports_the_field() {
        assert_eq!(
            parse_datetime("2026-09-07T08:00:00+08:00", "at")
                .unwrap()
                .to_rfc3339(),
            "2026-09-07T00:00:00+00:00"
        );
        assert_eq!(
            parse_datetime("invalid", "at").unwrap_err().to_string(),
            "at must be RFC 3339"
        );
    }
}
