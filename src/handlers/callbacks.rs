use teloxide::prelude::*;
use crate::utils::{encode, encode_file_id, get_random_emoji};
use crate::models::{StateStorage, set_user_state, get_user_state, clear_user_state, BotState};

pub async fn callback_handler(bot: Bot, q: CallbackQuery, state_storage: StateStorage) -> ResponseResult<()> {
    let user_id = q.from.id.0 as i64;
    let state = get_user_state(&state_storage, user_id).await;

    if let Some(data) = &q.data {
        let parts: Vec<&str> = data.split(':').collect();

        match parts[0] {
            "encode" if parts.len() == 2 => {
                let emoji = parts[1];
                // Check if we're encoding a file or text
                match state {
                    BotState::AwaitingFileEmoji { file_id, .. } => {
                        handle_file_encode(&bot, &q, emoji, &file_id, &state_storage, user_id).await?;
                    }
                    _ => {
                        // Try file first, fallback to text
                        handle_encode_with_file_check(&bot, &q, emoji, &state_storage, user_id).await?;
                    }
                }
            }
            "random" => {
                let emoji = get_random_emoji();
                // Check if we're encoding a file or text
                match state {
                    BotState::AwaitingFileEmoji { file_id, .. } => {
                        handle_file_encode(&bot, &q, emoji, &file_id, &state_storage, user_id).await?;
                    }
                    _ => {
                        // Try file first, fallback to text
                        handle_encode_with_file_check(&bot, &q, emoji, &state_storage, user_id).await?;
                    }
                }
            }
            "custom" => {
                handle_custom(&bot, &q, &state_storage).await?;
            }
            _ => {}
        }
    }

    bot.answer_callback_query(q.id).await?;
    Ok(())
}

async fn handle_file_encode(
    bot: &Bot,
    q: &CallbackQuery,
    emoji: &str,
    file_id: &str,
    state_storage: &StateStorage,
    user_id: i64,
) -> ResponseResult<()> {
    if let Some(msg) = q.message.as_ref().and_then(|m| m.regular_message()) {
        clear_user_state(state_storage, user_id).await;

        match encode_file_id(emoji, file_id) {
            Ok(encoded) => {
                bot.edit_message_text(msg.chat.id, msg.id, &encoded).await?;
            }
            Err(e) => {
                bot.send_message(msg.chat.id, format!("❌ Error encoding: {}", e))
                    .await?;
            }
        }
    }
    Ok(())
}

async fn handle_custom(bot: &Bot, q: &CallbackQuery, state_storage: &StateStorage) -> ResponseResult<()> {
    if let Some(msg) = q.message.as_ref().and_then(|m| m.regular_message()) {
        let user_id = q.from.id.0 as i64;
        let current_state = get_user_state(state_storage, user_id).await;

        // Check if we're in file encoding mode
        if let BotState::AwaitingFileEmoji { file_id, file_type } = current_state {
            // Keep the file info but wait for custom emoji
            let state = BotState::AwaitingFileEmoji {
                file_id,
                file_type,
            };
            set_user_state(state_storage, user_id, state).await;

            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                "Please send me the emoji you want to use for encoding:",
            )
            .await?;
        } else {
            // Original text encoding mode
            // Get the original text from the replied message
            if let Some(reply_to_msg) = msg.reply_to_message() {
                if let Some(text) = reply_to_msg.text() {
                    let state = BotState::AwaitingCustomEmoji {
                        text: text.to_string(),
                    };
                    set_user_state(state_storage, user_id, state).await;

                    bot.edit_message_text(
                        msg.chat.id,
                        msg.id,
                        "Please send me the emoji you want to use for encoding:",
                    )
                    .await?;
                } else {
                    bot.send_message(msg.chat.id, "❌ Could not find the original text message.")
                        .await?;
                }
            } else {
                bot.send_message(msg.chat.id, "❌ Could not find the original text message.")
                    .await?;
            }
        }
    }
    Ok(())
}

/// Handle encoding with automatic file/text detection from replied message
async fn handle_encode_with_file_check(
    bot: &Bot,
    q: &CallbackQuery,
    emoji: &str,
    state_storage: &StateStorage,
    user_id: i64,
) -> ResponseResult<()> {
    if let Some(msg) = q.message.as_ref().and_then(|m| m.regular_message()) {
        if let Some(reply_to_msg) = msg.reply_to_message() {
            // Check if it's a file first
            if let Some((file_id, _file_type)) = extract_file_info_from_callback(reply_to_msg) {
                handle_file_encode(bot, q, emoji, &file_id, state_storage, user_id).await?;
            } else if let Some(text) = reply_to_msg.text() {
                // It's a text message
                match encode(emoji, text) {
                    Ok(encoded) => {
                        bot.edit_message_text(msg.chat.id, msg.id, &encoded).await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, format!("❌ Error encoding: {}", e))
                            .await?;
                    }
                }
            } else {
                bot.send_message(msg.chat.id, "❌ Could not find the original message.")
                    .await?;
            }
        } else {
            bot.send_message(msg.chat.id, "❌ Could not find the original message.")
                .await?;
        }
    }
    Ok(())
}

/// Extract file_id and file_type from a message (for callbacks)
fn extract_file_info_from_callback(msg: &Message) -> Option<(String, String)> {
    if let Some(photo) = msg.photo() {
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
