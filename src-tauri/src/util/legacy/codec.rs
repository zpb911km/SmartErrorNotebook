use super::super::codec::decode_base64;

pub fn decode_legacy_bytes(value: &[u8]) -> Result<Vec<u8>, String> {
    let encoded = std::str::from_utf8(value)
        .map_err(|error| format!("attachment base64 data is not UTF-8: {error}"))?;
    decode_base64(encoded)
}

pub fn legacy_file_type(mime_type: &str) -> String {
    match mime_type {
        "image/jpeg" => "jpeg".to_owned(),
        "image/png" => "png".to_owned(),
        "image/gif" => "gif".to_owned(),
        "image/webp" => "webp".to_owned(),
        "image/svg+xml" => "svg".to_owned(),
        "application/pdf" => "pdf".to_owned(),
        other => other.to_owned(),
    }
}

pub fn mime_type(value: &[u8], legacy_hint: &str) -> String {
    if let Some(kind) = infer::get(value) {
        return kind.mime_type().to_owned();
    }

    let hint = legacy_hint.trim().to_ascii_lowercase();
    if hint.contains('/') {
        return hint;
    }
    match hint.as_str() {
        "jpg" | "jpeg" => "image/jpeg".to_owned(),
        "png" => "image/png".to_owned(),
        "gif" => "image/gif".to_owned(),
        "webp" => "image/webp".to_owned(),
        "svg" => "image/svg+xml".to_owned(),
        "pdf" => "application/pdf".to_owned(),
        _ => "application/octet-stream".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_legacy_base64_byte_arrays() {
        assert_eq!(decode_legacy_bytes(b"AQID").unwrap(), [1, 2, 3]);
    }

    #[test]
    fn maps_mime_types_to_legacy_file_types() {
        assert_eq!(legacy_file_type("image/jpeg"), "jpeg");
        assert_eq!(legacy_file_type("image/png"), "png");
        assert_eq!(
            legacy_file_type("application/octet-stream"),
            "application/octet-stream"
        );
    }

    #[test]
    fn normalizes_mime_types_from_content_and_legacy_hints() {
        assert_eq!(mime_type(&[0xff, 0xd8, 0xff, 0xd9], "image"), "image/jpeg");
        assert_eq!(mime_type(b"unknown", "jpeg"), "image/jpeg");
        assert_eq!(
            mime_type(b"unknown", "application/custom"),
            "application/custom"
        );
        assert_eq!(mime_type(b"unknown", "unknown"), "application/octet-stream");
    }
}
