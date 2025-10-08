use teloxide::prelude::*;
use teloxide::types::{InputFile, FileId};
use crate::models::{StateStorage, get_user_state, clear_user_state, BotState};
use crate::utils::{decode_with_file_check, encode, encode_file_id, decode_file_type, FileType};
use crate::handlers::commands::create_emoji_keyboard;

pub async fn message_handler(bot: Bot, msg: Message, state_storage: StateStorage) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
    let state = get_user_state(&state_storage, user_id).await;

    match state {
        BotState::AwaitingCustomEmoji { text: original_text } => {
            let text = msg.text().unwrap_or("").to_string();
            if !text.is_empty() {
                handle_custom_emoji(bot, msg, state_storage, user_id, &original_text, &text).await?;
            }
        }
        BotState::AwaitingFileEmoji { file_id, file_type } => {
            let text = msg.text().unwrap_or("").to_string();
            if !text.is_empty() {
                handle_file_custom_emoji(bot, msg, state_storage, user_id, &file_id, &file_type, &text).await?;
            }
        }
        BotState::Idle => {
            // Check if this is a file message (only in private chats)
            if matches!(msg.chat.kind, teloxide::types::ChatKind::Private(_)) {
                if let Some((file_id, file_type)) = extract_file_info(&msg) {
                    handle_file_message(bot, msg, state_storage, user_id, file_id, file_type).await?;
                    return Ok(());
                }
            }

            let text = msg.text().unwrap_or("").to_string();
            if text.is_empty() {
                return Ok(());
            }

            // Only decode messages in private chats
            if matches!(msg.chat.kind, teloxide::types::ChatKind::Private(_)) {
                // Try to decode only if the message contains variation selectors
                if text.chars().any(|c| {
                    let code = c as u32;
                    (0xFE00..=0xFE0F).contains(&code) || (0xE0100..=0xE01EF).contains(&code)
                }) {
                    if let Ok((is_file, content)) = decode_with_file_check(&text) {
                        if is_file {
                            // It's a file_id, try to send the file
                            handle_decode_file(&bot, &msg, &content).await?;
                            return Ok(());
                        } else if !content.is_empty() {
                            // It's regular text
                            bot.send_message(msg.chat.id, format!("🔓 Decoded message:\n\n{}", content))
                                .await?;
                            return Ok(());
                        }
                    }
                }
            }

            // Only show emoji keyboard in private chats
            // In groups/channels, only respond to commands or encoded messages
            if matches!(msg.chat.kind, teloxide::types::ChatKind::Private(_)) {
                let keyboard = create_emoji_keyboard();
                bot.send_message(
                    msg.chat.id,
                    "Select an emoji to encode your message:",
                )
                .reply_parameters(teloxide::types::ReplyParameters::new(msg.id))
                .reply_markup(keyboard)
                .await?;
            }
        }
    }

    Ok(())
}

/// Extract file_id and file_type from a message
/// Returns (file_id, file_type) if the message contains a supported file
fn extract_file_info(msg: &Message) -> Option<(String, String)> {
    if let Some(photo) = msg.photo() {
        // Get the largest photo size
        if let Some(largest) = photo.last() {
            return Some((largest.file.id.to_string(), "photo".to_string()));
        }
    }

    if let Some(video) = msg.video() {
        return Some((video.file.id.to_string(), "video".to_string()));
    }

    if let Some(audio) = msg.audio() {
        return Some((audio.file.id.to_string(), "audio".to_string()));
    }

    if let Some(document) = msg.document() {
        return Some((document.file.id.to_string(), "document".to_string()));
    }

    if let Some(sticker) = msg.sticker() {
        return Some((sticker.file.id.to_string(), "sticker".to_string()));
    }

    if let Some(voice) = msg.voice() {
        return Some((voice.file.id.to_string(), "voice".to_string()));
    }

    if let Some(video_note) = msg.video_note() {
        return Some((video_note.file.id.to_string(), "video note".to_string()));
    }

    if let Some(animation) = msg.animation() {
        return Some((animation.file.id.to_string(), "animation".to_string()));
    }

    None
}

/// Get human-readable file type name
fn get_file_type_name(file_type: &str) -> &str {
    match file_type {
        "photo" => "photo",
        "video" => "video",
        "audio" => "audio",
        "document" => "document",
        "sticker" => "sticker",
        "voice" => "voice message",
        "video note" => "video note",
        "animation" => "animation",
        _ => "file",
    }
}

/// Handle when a user sends a file
async fn handle_file_message(
    bot: Bot,
    msg: Message,
    _state_storage: StateStorage,
    _user_id: i64,
    _file_id: String,
    file_type: String,
) -> ResponseResult<()> {
    let keyboard = create_emoji_keyboard();
    let type_name = get_file_type_name(&file_type);

    // Don't set state - keep user in Idle so they can send multiple files
    // The callback handler will get file info from the replied message

    bot.send_message(
        msg.chat.id,
        format!("Select an emoji to hide your {}:", type_name),
    )
    .reply_parameters(teloxide::types::ReplyParameters::new(msg.id))
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

/// Handle when user sends custom emoji for a file
async fn handle_file_custom_emoji(
    bot: Bot,
    msg: Message,
    state_storage: StateStorage,
    user_id: i64,
    file_id: &str,
    _file_type: &str,
    emoji: &str,
) -> ResponseResult<()> {
    clear_user_state(&state_storage, user_id).await;

    match encode_file_id(emoji, file_id) {
        Ok(encoded) => {
            bot.send_message(msg.chat.id, &encoded).await?;
        }
        Err(e) => {
            bot.send_message(
                msg.chat.id,
                format!("❌ Error encoding: {}", e),
            )
            .await?;
        }
    }

    Ok(())
}

/// Handle decoding and sending files
async fn handle_decode_file(bot: &Bot, msg: &Message, file_id: &str) -> ResponseResult<()> {
    // Try to send the file. If it fails, fall back to showing the file_id as text
    let send_result = try_send_file(bot, msg.chat.id, file_id).await;

    match send_result {
        Ok(_) => {
            // File sent successfully
            Ok(())
        }
        Err(_) => {
            // Failed to send file, show the file_id as text instead
            bot.send_message(
                msg.chat.id,
                format!("🔓 Decoded file ID:\n\n`{}`\n\n⚠️ Unable to send this file. It may have been deleted or is no longer accessible.", file_id)
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
            Ok(())
        }
    }
}

/// Try to send a file by its file_id
/// Decodes the file_id to determine the type and sends it directly
async fn try_send_file(bot: &Bot, chat_id: ChatId, file_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Decode the file_id to get the file type
    let file_type = decode_file_type(file_id)?;

    // Send based on the decoded type
    match file_type {
        FileType::Photo => {
            bot.send_photo(chat_id, InputFile::file_id(FileId(file_id.to_string()))).await?;
        }
        FileType::Video => {
            bot.send_video(chat_id, InputFile::file_id(FileId(file_id.to_string()))).await?;
        }
        FileType::Voice => {
            bot.send_voice(chat_id, InputFile::file_id(FileId(file_id.to_string()))).await?;
        }
        FileType::Document => {
            bot.send_document(chat_id, InputFile::file_id(FileId(file_id.to_string()))).await?;
        }
        FileType::Sticker => {
            bot.send_sticker(chat_id, InputFile::file_id(FileId(file_id.to_string()))).await?;
        }
        FileType::Audio => {
            bot.send_audio(chat_id, InputFile::file_id(FileId(file_id.to_string()))).await?;
        }
        FileType::Animation => {
            bot.send_animation(chat_id, InputFile::file_id(FileId(file_id.to_string()))).await?;
        }
        FileType::VideoNote => {
            bot.send_video_note(chat_id, InputFile::file_id(FileId(file_id.to_string()))).await?;
        }
        FileType::Unknown => {
            return Err("Unknown file type".into());
        }
    }

    Ok(())
}

async fn handle_custom_emoji(
    bot: Bot,
    msg: Message,
    state_storage: StateStorage,
    user_id: i64,
    original_text: &str,
    emoji: &str,
) -> ResponseResult<()> {
    clear_user_state(&state_storage, user_id).await;

    match encode(emoji, original_text) {
        Ok(encoded) => {
            bot.send_message(msg.chat.id, &encoded).await?;
        }
        Err(e) => {
            bot.send_message(
                msg.chat.id,
                format!("❌ Error encoding: {}", e),
            )
            .await?;
        }
    }

    Ok(())
}
