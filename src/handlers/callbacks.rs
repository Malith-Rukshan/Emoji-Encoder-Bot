use teloxide::prelude::*;
use crate::utils::{encode, get_random_emoji};
use crate::models::{StateStorage, set_user_state, BotState};

pub async fn callback_handler(bot: Bot, q: CallbackQuery, state_storage: StateStorage) -> ResponseResult<()> {
    if let Some(data) = &q.data {
        let parts: Vec<&str> = data.splitn(3, ':').collect();

        match parts[0] {
            "encode" if parts.len() == 3 => {
                let emoji = parts[1];
                let text = parts[2];
                handle_encode(&bot, &q, emoji, text).await?;
            }
            "random" if parts.len() == 2 => {
                let text = parts[1];
                let emoji = get_random_emoji();
                handle_encode(&bot, &q, emoji, text).await?;
            }
            "custom" if parts.len() == 2 => {
                let text = parts[1];
                handle_custom(&bot, &q, &state_storage, text).await?;
            }
            _ => {}
        }
    }

    bot.answer_callback_query(q.id).await?;
    Ok(())
}

async fn handle_encode(bot: &Bot, q: &CallbackQuery, emoji: &str, text: &str) -> ResponseResult<()> {
    if let Some(msg) = &q.message {
        match encode(emoji, text) {
            Ok(encoded) => {
                bot.edit_message_text(msg.chat().id, msg.id(), &encoded).await?;
            }
            Err(e) => {
                bot.send_message(msg.chat().id, format!("❌ Error encoding: {}", e))
                    .await?;
            }
        }
    }
    Ok(())
}

async fn handle_custom(bot: &Bot, q: &CallbackQuery, state_storage: &StateStorage, text: &str) -> ResponseResult<()> {
    if let Some(msg) = &q.message {
        let user_id = q.from.id.0 as i64;

        let state = BotState::AwaitingCustomEmoji {
            text: text.to_string(),
        };
        set_user_state(state_storage, user_id, state).await;

        bot.edit_message_text(
            msg.chat().id,
            msg.id(),
            "Please send me the emoji you want to use for encoding:",
        )
        .await?;
    }
    Ok(())
}
