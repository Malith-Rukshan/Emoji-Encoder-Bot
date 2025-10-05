use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup}};
use crate::utils::{EMOJI_LIST, encode, decode, get_random_emoji};
use crate::models::DbClient;

pub async fn start_handler(bot: Bot, msg: Message, db: DbClient) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let chat_type = format!("{:?}", msg.chat.kind).to_lowercase();
    let title = msg.chat.title().map(|s| s.to_string());
    let username = msg.chat.username().map(|s| s.to_string());

    db.save_chat(chat_id, chat_type, title, username).await.ok();

    let bot_username = bot.get_me().await?.username.clone().unwrap_or_else(|| "EmojiEncoderBot".to_string());

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::url(
                "➕ Add to Channel ➕",
                format!("https://t.me/{}?startchannel=botstart", bot_username).parse().unwrap()
            ),
            InlineKeyboardButton::url(
                "➕ Add to Group ➕",
                format!("https://t.me/{}?startgroup=botstart", bot_username).parse().unwrap()
            ),
        ],
        vec![
            InlineKeyboardButton::url(
                "📥 GitHub Source",
                "https://github.com/Malith-Rukshan/Emoji-Encoder-Bot".parse().unwrap()
            ),
        ],
    ]);

    bot.send_message(
        msg.chat.id,
        "👋 *Welcome to Emoji Encoder Bot\\!*\n\n\
         Send me any text and I'll help you hide it inside an emoji using Unicode variation selectors\\.\n\n\
         📝 *Features:*\n\
         • Send text to encode with an emoji\n\
         • Send encoded emoji to decode the hidden message\n\
         • Use /encode or /decode commands \\(works in groups\\)\n\
         • Use inline mode: @EmojiEncoderBot <emoji> <text>\n\n\
         Just send me a message to get started\\!"
    )
    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

pub async fn about_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    let about_text = "🔐 *Emoji Encoder Bot*\n\n\
        *Hide Secret Messages Inside Emojis\\!* 🤫✨\n\n\
        A lightweight Telegram bot built with Rust that uses Unicode variation selectors to hide text messages inside emojis\\. \
        The encoded emoji looks completely normal but contains hidden data\\!\n\n\
        🦀 *Built with:*\n\
        • Rust \\- High\\-performance systems language\n\
        • Teloxide \\- Elegant Telegram bot framework\n\
        • MongoDB \\- Optional stats tracking\n\n\
        🔬 *How it works:*\n\
        The bot encodes your text into UTF\\-8 bytes, then converts each byte to invisible Unicode variation selectors \\(VS1\\-VS256\\) that are appended to the emoji\\. \
        These invisible characters are completely transparent to the user but can be decoded back to the original text\\!\n\n\
        📊 *Open Source:*\n\
        This bot is fully open source and available on GitHub\\. Feel free to contribute, report issues, or deploy your own instance\\!\n\n\
        https://github\\.com/Malith\\-Rukshan/Emoji\\-Encoder\\-Bot\n\n\
        👨‍💻 *Developer:* @MalithRukshan\n\n\
        Made with ❤️ and Rust 🦀";

    bot.send_message(msg.chat.id, about_text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
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

pub fn create_emoji_keyboard() -> InlineKeyboardMarkup {
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
                    format!("encode:{}", emoji),
                ));
            }
        }
        keyboard.push(button_row);
    }

    // Add Random button
    keyboard.push(vec![InlineKeyboardButton::callback(
        "🎲 Random",
        "random".to_string(),
    )]);

    // Add Custom button
    keyboard.push(vec![InlineKeyboardButton::callback(
        "✏️ Custom Emoji",
        "custom".to_string(),
    )]);

    InlineKeyboardMarkup::new(keyboard)
}

pub async fn encode_command_handler(bot: Bot, msg: Message, text: String) -> ResponseResult<()> {
    let text_to_encode = if text.trim().is_empty() {
        // If no text provided, check if it's a reply to a message
        if let Some(reply_msg) = msg.reply_to_message() {
            reply_msg.text().unwrap_or("").to_string()
        } else {
            bot.send_message(msg.chat.id, "❌ Please provide text to encode or reply to a message with /encode")
                .await?;
            return Ok(());
        }
    } else {
        text
    };

    if text_to_encode.is_empty() {
        bot.send_message(msg.chat.id, "❌ No text to encode")
            .await?;
        return Ok(());
    }

    let emoji = get_random_emoji();
    match encode(emoji, &text_to_encode) {
        Ok(encoded) => {
            bot.send_message(msg.chat.id, &encoded).await?;
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error encoding: {}", e))
                .await?;
        }
    }

    Ok(())
}

pub async fn decode_command_handler(bot: Bot, msg: Message, text: String) -> ResponseResult<()> {
    let text_to_decode = if text.trim().is_empty() {
        // If no text provided, check if it's a reply to a message
        if let Some(reply_msg) = msg.reply_to_message() {
            reply_msg.text().unwrap_or("").to_string()
        } else {
            bot.send_message(msg.chat.id, "❌ Please provide encoded emoji to decode or reply to a message with /decode")
                .await?;
            return Ok(());
        }
    } else {
        text
    };

    if text_to_decode.is_empty() {
        bot.send_message(msg.chat.id, "❌ No text to decode")
            .await?;
        return Ok(());
    }

    // Check if the text contains variation selectors
    if !text_to_decode.chars().any(|c| {
        let code = c as u32;
        (0xFE00..=0xFE0F).contains(&code) || (0xE0100..=0xE01EF).contains(&code)
    }) {
        bot.send_message(msg.chat.id, "❌ No encoded message found")
            .await?;
        return Ok(());
    }

    match decode(&text_to_decode) {
        Ok(decoded) => {
            if decoded.is_empty() {
                bot.send_message(msg.chat.id, "❌ No encoded message found")
                    .await?;
            } else {
                bot.send_message(msg.chat.id, format!("🔓 Decoded message:\n\n{}", decoded))
                    .await?;
            }
        }
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ Error decoding: {}", e))
                .await?;
        }
    }

    Ok(())
}
