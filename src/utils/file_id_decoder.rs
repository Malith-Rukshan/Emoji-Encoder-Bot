/// Telegram File ID decoder
/// Based on https://github.com/luckydonald/telegram_file_id
///
/// This module decodes Telegram file_ids to extract the file type.
/// File IDs are base64url encoded and RLE (run-length) encoded.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

/// File type constants from Telegram
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Photo,
    Video,
    Voice,
    Document,
    Sticker,
    Audio,
    Animation,
    VideoNote,
    Unknown,
}

const TYPE_ID_FILE_REFERENCE_FLAG: u32 = 1 << 25;
const TYPE_ID_WEB_LOCATION_FLAG: u32 = 1 << 24;

// Type IDs from Telegram
const TYPE_THUMBNAIL: u32 = 0;
const TYPE_PROFILE_PHOTO: u32 = 1;
const TYPE_PHOTO: u32 = 2;
const TYPE_VOICE: u32 = 3;
const TYPE_VIDEO: u32 = 4;
const TYPE_DOCUMENT: u32 = 5;
const TYPE_STICKER: u32 = 8;
const TYPE_AUDIO: u32 = 9;
const TYPE_ANIMATION: u32 = 10;
const TYPE_VIDEO_NOTE: u32 = 13;

/// Decode a Telegram file_id and extract the file type
pub fn decode_file_type(file_id: &str) -> Result<FileType, Box<dyn std::error::Error + Send + Sync>> {
    // Step 1: Base64 URL-safe decode
    let decoded = URL_SAFE_NO_PAD.decode(file_id)?;

    // Step 2: RLE decode
    let rle_decoded = rle_decode(&decoded);

    // Step 3: Read first 4 bytes as little-endian u32 (type_id)
    if rle_decoded.len() < 4 {
        return Err("File ID too short".into());
    }

    let type_id = u32::from_le_bytes([
        rle_decoded[0],
        rle_decoded[1],
        rle_decoded[2],
        rle_decoded[3],
    ]);

    // Step 4: Normalize type_id (remove flags)
    let normalized_type_id = normalize_type_id(type_id);

    // Step 5: Map to FileType
    Ok(match normalized_type_id {
        TYPE_PHOTO | TYPE_THUMBNAIL | TYPE_PROFILE_PHOTO => FileType::Photo,
        TYPE_VIDEO => FileType::Video,
        TYPE_VOICE => FileType::Voice,
        TYPE_DOCUMENT => FileType::Document,
        TYPE_STICKER => FileType::Sticker,
        TYPE_AUDIO => FileType::Audio,
        TYPE_ANIMATION => FileType::Animation,
        TYPE_VIDEO_NOTE => FileType::VideoNote,
        _ => FileType::Unknown,
    })
}

/// Normalize type_id by removing flag bits
fn normalize_type_id(type_id: u32) -> u32 {
    type_id & !TYPE_ID_FILE_REFERENCE_FLAG & !TYPE_ID_WEB_LOCATION_FLAG
}

/// RLE decode (run-length encoding decode)
/// Based on Telegram's implementation
fn rle_decode(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < data.len() {
        if data[i] == 0 {
            // Zero byte indicates a run
            if i + 1 < data.len() {
                // Next byte is the count of zeros to insert
                let count = data[i + 1] as usize;
                result.extend(vec![0; count]);
                i += 2;
            } else {
                break;
            }
        } else {
            // Regular byte, just copy it
            result.push(data[i]);
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_decode() {
        // Test basic RLE decoding
        let encoded = vec![1, 2, 0, 3, 4, 5];
        let decoded = rle_decode(&encoded);
        assert_eq!(decoded, vec![1, 2, 0, 0, 0, 4, 5]);
    }

    #[test]
    fn test_normalize_type_id() {
        // Test with flags
        let type_id_with_flags = TYPE_PHOTO | TYPE_ID_FILE_REFERENCE_FLAG;
        assert_eq!(normalize_type_id(type_id_with_flags), TYPE_PHOTO);

        let type_id_with_both_flags = TYPE_STICKER | TYPE_ID_FILE_REFERENCE_FLAG | TYPE_ID_WEB_LOCATION_FLAG;
        assert_eq!(normalize_type_id(type_id_with_both_flags), TYPE_STICKER);
    }
}
