use teloxide::{prelude::*, types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile, FileId}};
use crate::utils::{EMOJI_LIST, encode, encode_file_id, decode_with_file_check, get_random_emoji, decode_file_type, FileType};
use crate::models::DbClient;

pub async fn start_handler(bot: Bot, msg: Message, db: DbClient) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let chat_type = match msg.chat.kind {
        teloxide::types::ChatKind::Public(ref public) => match public.kind {
            teloxide::types::PublicChatKind::Channel(_) => "channel".to_string(),
            teloxide::types::PublicChatKind::Group => "group".to_string(),
            teloxide::types::PublicChatKind::Supergroup(_) => "supergroup".to_string(),
        },
        teloxide::types::ChatKind::Private(_) => "private".to_string(),
    };
    let title = msg.chat.title().map(|s| s.to_string());
    let username = msg.chat.username().map(|s| s.to_string());

    db.save_chat(chat_id, chat_type, title, username).await.ok();

    let bot_username = bot.get_me().await?.username.clone().unwrap_or_else(|| "EmojiEncoderBot".to_string());

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::switch_inline_query(
                "🔄 Try Inline Mode",
                ""
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
         Send me any text or file and I'll help you hide it inside an emoji using Unicode variation selectors\\.\n\n\
         📝 *Features:*\n\
         • Hide text messages inside emojis\n\
         • Hide files \\(photos, videos, stickers, documents, etc\\.\\) inside emojis\n\
         • Send encoded emoji to reveal the hidden content\n\
         • Use /encode or /decode commands \\(works in groups\\)\n\
         • Use inline mode: @EmojiEncoderBot [emoji] [text]\n\n\
         Just send me a message or file to get started\\!"
    )
    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
    .reply_markup(keyboard)
    .await?;
    Ok(())
}

pub async fn help_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    let help_text = "📚 *How to Use Emoji Encoder Bot*\n\n\
        *In Private Chat:*\n\
        • Send any text and select an emoji to encode\n\
        • Send any file \\(photo, video, sticker, etc\\.\\) and select an emoji to hide it\n\
        • Send encoded emoji to automatically decode and reveal content\n\n\
        *Supported File Types:*\n\
        📷 Photos • 🎬 Videos • 🎵 Audio • 📄 Documents\n\
        🎭 Stickers • 🎤 Voice • 🎥 Video Notes • 🎞️ Animations\n\n\
        *Commands:*\n\
        /encode \\<text\\> \\- Encode text with random emoji\n\
        /encode \\(reply\\) \\- Encode replied message or file\n\
        /decode \\<emoji\\> \\- Decode hidden message or file\n\
        /decode \\(reply\\) \\- Decode replied message\n\n\
        *In Groups:*\n\
        Use /encode or /decode commands with text or as reply to messages/files\\.\n\n\
        *Inline Mode:*\n\
        Type @EmojiEncoderBot followed by your text in any chat\\.\n\n\
        *Other Commands:*\n\
        /start \\- Start the bot\n\
        /help \\- Show this help message\n\
        /about \\- About this bot\n\n\
        🎬 Demo Video: [Click Here](https://raw.githubusercontent.com/Malith-Rukshan/Emoji-Encoder-Bot/refs/heads/main/assets/demo.mp4)";

    bot.send_message(msg.chat.id, help_text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
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
        👨‍💻 *Developer:* @MalithRukshan";

    bot.send_message(msg.chat.id, about_text)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;
    Ok(())
}

pub async fn stats_handler(bot: Bot, msg: Message, db: DbClient, admin_ids: Vec<i64>) -> ResponseResult<()> {
    use sysinfo::{System, Pid, ProcessesToUpdate};

    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    if !admin_ids.contains(&user_id) {
        bot.send_message(msg.chat.id, "❌ This command is only available for administrators.")
            .await?;
        return Ok(());
    }

    // Get this bot process's stats
    let mut sys = System::new();
    let pid = Pid::from_u32(std::process::id());
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

    let (cpu_usage, memory_mb) = if let Some(process) = sys.process(pid) {
        let cpu = process.cpu_usage();
        let mem = process.memory() as f64 / 1024.0 / 1024.0; // Convert to MB
        (cpu, mem)
    } else {
        (0.0, 0.0)
    };

    match db.get_stats().await {
        Ok(stats) => {
            let message = format!(
                "📊 *Bot Statistics*\n\n\
                 *Database Stats:*\n\
                 👥 Total Chats: `{}`\n\
                 🧑 Users: `{}`\n\
                 👥 Groups: `{}`\n\
                 📢 Channels: `{}`\n\n\
                 *Bot Process:*\n\
                 🧠 CPU Usage: `{:.1}\\%`\n\
                 💾 Memory: `{:.1}MB`",
                stats.total_chats,
                stats.users,
                stats.groups,
                stats.channels,
                cpu_usage,
                memory_mb
            ).replace('.', "\\.");
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
    // Check if replying to a file message
    if text.trim().is_empty() {
        if let Some(reply_msg) = msg.reply_to_message() {
            // Try to extract file info from the replied message
            if let Some((file_id, _file_type)) = extract_file_info_from_msg(reply_msg) {
                // It's a file, encode the file_id
                let emoji = get_random_emoji();
                match encode_file_id(emoji, &file_id) {
                    Ok(encoded) => {
                        bot.send_message(msg.chat.id, &encoded).await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, format!("❌ Error encoding: {}", e))
                            .await?;
                    }
                }
                return Ok(());
            }

            // It's a text message
            let text_to_encode = reply_msg.text().unwrap_or("").to_string();
            if text_to_encode.is_empty() {
                bot.send_message(msg.chat.id, "❌ No text or file to encode")
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
            return Ok(());
        } else {
            bot.send_message(msg.chat.id, "❌ Please provide text to encode or reply to a message with /encode")
                .await?;
            return Ok(());
        }
    }

    // Text provided directly in command
    let emoji = get_random_emoji();
    match encode(emoji, &text) {
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

    match decode_with_file_check(&text_to_decode) {
        Ok((is_file, content)) => {
            if content.is_empty() {
                bot.send_message(msg.chat.id, "❌ No encoded message found")
                    .await?;
            } else if is_file {
                // It's a file_id, try to send the file
                handle_decode_file_command(&bot, &msg, &content).await?;
            } else {
                // It's regular text
                bot.send_message(msg.chat.id, format!("🔓 Decoded message:\n\n{}", content))
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

/// Extract file_id and file_type from a message
fn extract_file_info_from_msg(msg: &Message) -> Option<(String, String)> {
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

/// Handle decoding and sending files in command context
async fn handle_decode_file_command(bot: &Bot, msg: &Message, file_id: &str) -> ResponseResult<()> {
    let send_result = try_send_file_command(bot, msg.chat.id, file_id).await;

    match send_result {
        Ok(_) => Ok(()),
        Err(_) => {
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

/// Try to send a file by its file_id (for commands)
/// Decodes the file_id to determine the type and sends it directly
async fn try_send_file_command(bot: &Bot, chat_id: ChatId, file_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
