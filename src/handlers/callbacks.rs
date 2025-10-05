use teloxide::prelude::*;
use crate::utils::{encode, get_random_emoji};
use crate::models::{StateStorage, set_user_state, BotState};

pub async fn callback_handler(bot: Bot, q: CallbackQuery, state_storage: StateStorage) -> ResponseResult<()> {
    if let Some(data) = &q.data {
        let parts: Vec<&str> = data.split(':').collect();

        match parts[0] {
            "encode" if parts.len() == 2 => {
                let emoji = parts[1];
                handle_encode(&bot, &q, emoji).await?;
            }
            "random" => {
                let emoji = get_random_emoji();
                handle_encode(&bot, &q, emoji).await?;
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

async fn handle_custom(bot: &Bot, q: &CallbackQuery, state_storage: &StateStorage) -> ResponseResult<()> {
    if let Some(msg) = q.message.as_ref().and_then(|m| m.regular_message()) {
        let user_id = q.from.id.0 as i64;

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
    Ok(())
}
