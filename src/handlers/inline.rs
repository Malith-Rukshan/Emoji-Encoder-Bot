use teloxide::{
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText,
    },
};
use crate::utils::{encode, get_random_emoji, EMOJI_LIST};

pub async fn inline_query_handler(bot: Bot, q: InlineQuery) -> ResponseResult<()> {
    let query = q.query.trim();

    if query.is_empty() {
        bot.answer_inline_query(q.id.clone(), vec![]).await?;
        return Ok(());
    }

    // Parse query: first character(s) might be emoji, rest is text
    // Format: <emoji> <text> or just <text> (use default emoji)
    let (emoji, text) = parse_inline_query(query);

    let mut results: Vec<InlineQueryResult> = Vec::new();

    // If emoji was specified, add it FIRST
    if !emoji.is_empty() && emoji != "😀" {
        if let Ok(encoded) = encode(&emoji, &text) {
            results.push(create_inline_result(
                format!("selected_{}", emoji),
                &emoji,
                &text,
                &encoded,
            ));
        }
    }

    // Add random option at the top (after selected emoji)
    let random_emoji = get_random_emoji();
    if let Ok(encoded) = encode(random_emoji, &text) {
        results.push(create_inline_result("random".to_string(), random_emoji, &text, &encoded));
    }

    // Create results for each predefined emoji
    for (idx, &preset_emoji) in EMOJI_LIST.iter().enumerate() {
        // Skip if it's the same as the selected emoji
        if !emoji.is_empty() && emoji != "😀" && preset_emoji == emoji {
            continue;
        }

        if let Ok(encoded) = encode(preset_emoji, &text) {
            results.push(create_inline_result(
                format!("emoji_{}", idx),
                preset_emoji,
                &text,
                &encoded,
            ));
        }

        // Limit to 50 results (Telegram's limit)
        if results.len() >= 50 {
            break;
        }
    }

    bot.answer_inline_query(q.id.clone(), results).await?;
    Ok(())
}

fn parse_inline_query(query: &str) -> (String, String) {
    let chars: Vec<char> = query.chars().collect();

    if chars.is_empty() {
        return ("😀".to_string(), String::new());
    }

    // Check if first character is an emoji
    let first_char = chars[0];
    if is_emoji_char(first_char) {
        let emoji = chars[0].to_string();
        let text = if chars.len() > 1 {
            chars[1..].iter().collect::<String>().trim().to_string()
        } else {
            String::new()
        };
        return (emoji, text);
    }

    ("😀".to_string(), query.to_string())
}

fn is_emoji_char(c: char) -> bool {
    let code = c as u32;
    (0x1F300..=0x1F9FF).contains(&code) ||
    (0x2600..=0x27BF).contains(&code) ||
    (0x1F600..=0x1F64F).contains(&code)
}

fn create_inline_result(
    id: String,
    emoji: &str,
    text: &str,
    encoded: &str,
) -> InlineQueryResult {
    InlineQueryResult::Article(InlineQueryResultArticle::new(
        id,
        format!("{} {}", emoji, text),
        InputMessageContent::Text(InputMessageContentText::new(encoded)),
    )
    .description(format!("Encode with {}", emoji)))
}
