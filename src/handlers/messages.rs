use teloxide::prelude::*;
use crate::models::{StateStorage, get_user_state, clear_user_state, BotState};
use crate::utils::{decode, encode};
use crate::handlers::commands::create_emoji_keyboard;

pub async fn message_handler(bot: Bot, msg: Message, state_storage: StateStorage) -> ResponseResult<()> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
    let text = msg.text().unwrap_or("").to_string();

    if text.is_empty() {
        return Ok(());
    }

    let state = get_user_state(&state_storage, user_id).await;

    match state {
        BotState::AwaitingCustomEmoji { text: original_text } => {
            handle_custom_emoji(bot, msg, state_storage, user_id, &original_text, &text).await?;
        }
        BotState::Idle => {
            // Try to decode only if the message contains variation selectors
            if text.chars().any(|c| {
                let code = c as u32;
                (0xFE00..=0xFE0F).contains(&code) || (0xE0100..=0xE01EF).contains(&code)
            }) {
                if let Ok(decoded) = decode(&text) {
                    if !decoded.is_empty() {
                        bot.send_message(msg.chat.id, format!("🔓 Decoded message:\n\n{}", decoded))
                            .await?;
                        return Ok(());
                    }
                }
            }

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
