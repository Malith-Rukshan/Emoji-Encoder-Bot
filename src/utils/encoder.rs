use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncoderError {
    #[error("Invalid byte value: {0}")]
    InvalidByte(u8),
    #[error("UTF-8 encoding error")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

// Variation selectors block https://unicode.org/charts/nameslist/n_FE00.html
// VS1..=VS16
const VARIATION_SELECTOR_START: u32 = 0xFE00;
const VARIATION_SELECTOR_END: u32 = 0xFE0F;

// Variation selectors supplement https://unicode.org/charts/nameslist/n_E0100.html
// VS17..=VS256
const VARIATION_SELECTOR_SUPPLEMENT_START: u32 = 0xE0100;
const VARIATION_SELECTOR_SUPPLEMENT_END: u32 = 0xE01EF;

/// Convert a byte (0-255) to a variation selector character
fn to_variation_selector(byte: u8) -> Result<char, EncoderError> {
    if byte < 16 {
        char::from_u32(VARIATION_SELECTOR_START + byte as u32)
            .ok_or(EncoderError::InvalidByte(byte))
    } else {
        char::from_u32(VARIATION_SELECTOR_SUPPLEMENT_START + (byte - 16) as u32)
            .ok_or(EncoderError::InvalidByte(byte))
    }
}

/// Convert a variation selector character back to a byte
fn from_variation_selector(code_point: u32) -> Option<u8> {
    if (VARIATION_SELECTOR_START..=VARIATION_SELECTOR_END).contains(&code_point) {
        Some((code_point - VARIATION_SELECTOR_START) as u8)
    } else if (VARIATION_SELECTOR_SUPPLEMENT_START..=VARIATION_SELECTOR_SUPPLEMENT_END)
        .contains(&code_point)
    {
        Some((code_point - VARIATION_SELECTOR_SUPPLEMENT_START + 16) as u8)
    } else {
        None
    }
}

/// Encode text into an emoji by appending invisible variation selectors
pub fn encode(emoji: &str, text: &str) -> Result<String, EncoderError> {
    let bytes = text.as_bytes();
    let mut encoded = String::from(emoji);

    for &byte in bytes {
        let selector = to_variation_selector(byte)?;
        encoded.push(selector);
    }

    Ok(encoded)
}

/// Decode hidden text from an emoji with variation selectors
pub fn decode(text: &str) -> Result<String, EncoderError> {
    let mut decoded_bytes = Vec::new();

    for ch in text.chars() {
        if let Some(byte) = from_variation_selector(ch as u32) {
            decoded_bytes.push(byte);
        } else if !decoded_bytes.is_empty() {
            // Stop when we encounter a non-variation selector after starting to decode
            break;
        }
        // Skip the emoji itself at the beginning
    }

    String::from_utf8(decoded_bytes).map_err(EncoderError::from)
}

/// Encode a Telegram file_id into an emoji
pub fn encode_file_id(emoji: &str, file_id: &str) -> Result<String, EncoderError> {
    let file_data = format!("TG_FILE_{}", file_id.trim());
    encode(emoji, &file_data)
}

/// Decode and check if the decoded text is a file_id
/// Returns (is_file, content) where:
/// - is_file: true if it's a file_id, false if it's regular text
/// - content: the file_id (without TG_FILE_ prefix) or the decoded text
pub fn decode_with_file_check(text: &str) -> Result<(bool, String), EncoderError> {
    let decoded = decode(text)?;

    // Check if TG_FILE_ exists anywhere in the decoded text
    if let Some(pos) = decoded.find("TG_FILE_") {
        // Extract everything after TG_FILE_
        let file_part = &decoded[pos + 8..]; // 8 is the length of "TG_FILE_"

        // Remove all whitespace from the file_id
        let file_id = file_part.chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        if !file_id.is_empty() {
            return Ok((true, file_id));
        }
    }

    // Not a file, return the original decoded text
    Ok((false, decoded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let emoji = "😀";
        let text = "Hello, World!";

        let encoded = encode(emoji, text).unwrap();
        assert!(encoded.starts_with(emoji));

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_unicode_text() {
        let emoji = "🚀";
        let text = "Hello 世界 🌍";

        let encoded = encode(emoji, text).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_empty_text() {
        let emoji = "👍";
        let text = "";

        let encoded = encode(emoji, text).unwrap();
        assert_eq!(encoded, emoji);

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, text);
    }
}
