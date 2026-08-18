use base64::{engine::general_purpose, Engine};
use sha2::{Digest, Sha256};
use std::fmt::Write;

pub fn decode_base64(value: &str) -> Result<Vec<u8>, String> {
    let payload = value
        .trim()
        .strip_prefix("data:")
        .and_then(|data_url| data_url.split_once(',').map(|(_, payload)| payload))
        .unwrap_or_else(|| value.trim());
    let compact: String = payload.chars().filter(|c| !c.is_whitespace()).collect();

    general_purpose::STANDARD
        .decode(&compact)
        .or_else(|_| general_purpose::STANDARD_NO_PAD.decode(&compact))
        .map_err(|error| format!("invalid base64 attachment data: {error}"))
}

pub fn encode_base64(value: &[u8]) -> String {
    general_purpose::STANDARD.encode(value)
}

pub fn sha256(value: &[u8]) -> String {
    let mut output = String::with_capacity(64);
    for byte in Sha256::digest(value) {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_data_urls_and_unpadded_base64() {
        assert_eq!(
            decode_base64("data:image/png;base64,AQID").unwrap(),
            [1, 2, 3]
        );
        assert_eq!(decode_base64("AQI").unwrap(), [1, 2]);
    }

    #[test]
    fn computes_digest() {
        assert_eq!(
            sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
