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
                        handle_encode(&bot, &q, emoji).await?;
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
                        handle_encode(&bot, &q, emoji).await?;
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

async fn handle_encode(bot: &Bot, q: &CallbackQuery, emoji: &str) -> ResponseResult<()> {
    if let Some(msg) = q.message.as_ref().and_then(|m| m.regular_message()) {
        // Get the original text from the replied message
        if let Some(reply_to_msg) = msg.reply_to_message() {
            if let Some(text) = reply_to_msg.text() {
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
                bot.send_message(msg.chat.id, "❌ Could not find the original text message.")
                    .await?;
            }
        } else {
            bot.send_message(msg.chat.id, "❌ Could not find the original text message.")
                .await?;
        }
    }
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
