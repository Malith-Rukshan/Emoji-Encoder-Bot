use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup}};
use crate::utils::EMOJI_LIST;
use crate::models::DbClient;

pub async fn start_handler(bot: Bot, msg: Message, db: DbClient) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let chat_type = format!("{:?}", msg.chat.kind).to_lowercase();
    let title = msg.chat.title().map(|s| s.to_string());
    let username = msg.chat.username().map(|s| s.to_string());

    db.save_chat(chat_id, chat_type, title, username).await.ok();

    bot.send_message(
        msg.chat.id,
        "👋 Welcome to Emoji Encoder Bot!\n\n\
         Send me any text and I'll help you hide it inside an emoji using Unicode variation selectors.\n\n\
         📝 Features:\n\
         • Send text to encode with an emoji\n\
         • Send encoded emoji to decode the hidden message\n\
         • Use inline mode: @EmojiEncoderBot <emoji> <text>\n\n\
         Just send me a message to get started!"
    )
    .await?;
    Ok(())
}

pub async fn stats_handler(bot: Bot, msg: Message, db: DbClient, admin_ids: Vec<i64>) -> ResponseResult<()> {
    use sysinfo::System;

    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !admin_ids.contains(&user_id) {
        bot.send_message(msg.chat.id, "❌ This command is only available for administrators.")
            .await?;
        return Ok(());
    }

    // Get system stats
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0; // MB
    let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0; // MB
    let cpu_usage = sys.global_cpu_usage();

    match db.get_stats().await {
        Ok(stats) => {
            let message = format!(
                "📊 *Bot Statistics*\n\n\
                 *Database Stats:*\n\
                 👥 Total Chats: `{}`\n\
                 🧑 Users: `{}`\n\
                 👥 Groups: `{}`\n\
                 📢 Channels: `{}`\n\n\
                 *System Stats:*\n\
                 🧠 CPU Usage: `{:.1}%`\n\
                 💾 RAM: `{:.1} MB / {:.1} MB` ({:.1}%)",
                stats.total_chats,
                stats.users,
                stats.groups,
                stats.channels,
                cpu_usage,
                used_memory,
                total_memory,
                (used_memory / total_memory) * 100.0
            );
            bot.send_message(msg.chat.id, message)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error fetching stats: {}", e))
                .await?;
        }
    }

    Ok(())
}

pub fn create_emoji_keyboard(text: &str) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    // Create 5x3 grid of emojis (15 emojis total)
    let emojis_to_show = std::cmp::min(EMOJI_LIST.len(), 15);
    for row in 0..3 {
        let mut button_row = Vec::new();
        for col in 0..5 {
            let idx = row * 5 + col;
            if idx < emojis_to_show {
                let emoji = EMOJI_LIST[idx];
                button_row.push(InlineKeyboardButton::callback(
                    emoji.to_string(),
                    format!("encode:{}:{}", emoji, text),
                ));
            }
        }
        keyboard.push(button_row);
    }

    // Add Random button
    keyboard.push(vec![InlineKeyboardButton::callback(
        "🎲 Random",
        format!("random:{}", text),
    )]);

    // Add Custom button
    keyboard.push(vec![InlineKeyboardButton::callback(
        "✏️ Custom Emoji",
        format!("custom:{}", text),
    )]);

    InlineKeyboardMarkup::new(keyboard)
}
