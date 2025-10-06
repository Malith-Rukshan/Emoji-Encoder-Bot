mod handlers;
mod models;
mod utils;

use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use handlers::{start_handler, stats_handler, about_handler, message_handler, callback_handler, inline_query_handler};
use models::{DbClient, create_state_storage};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting Emoji Encoder Bot...");

    dotenvy::dotenv().ok();

    let bot_token = std::env::var("BOT_TOKEN")
        .expect("BOT_TOKEN must be set");
    let bot = Bot::new(bot_token);

    let mongodb_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| {
            log::warn!("MONGODB_URI not set, running without database (stats disabled)");
            String::new()
        });

    let admin_ids: Vec<i64> = std::env::var("ADMIN_IDS")
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let db = if !mongodb_uri.is_empty() {
        match DbClient::new(&mongodb_uri).await {
            Ok(db) => {
                log::info!("MongoDB connected successfully");
                Some(db)
            }
            Err(e) => {
                log::error!("Failed to connect to MongoDB: {}", e);
                None
            }
        }
    } else {
        None
    };

    let state_storage = create_state_storage();

    log::info!("Bot started successfully!");

    let handler = dptree::entry()
        .branch(Update::filter_message().branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint({
                    let db = db.clone();
                    let admin_ids = admin_ids.clone();
                    move |bot: Bot, msg: Message, cmd: Command| {
                        let db = db.clone();
                        let admin_ids = admin_ids.clone();
                        async move {
                            command_handler(bot, msg, cmd, db, admin_ids).await
                        }
                    }
                })
        ).branch({
            let state_storage = state_storage.clone();
            dptree::endpoint(move |bot: Bot, msg: Message| {
                let state_storage = state_storage.clone();
                async move {
                    message_handler(bot, msg, state_storage).await
                }
            })
        }))
        .branch(Update::filter_callback_query().endpoint({
            let state_storage = state_storage.clone();
            move |bot: Bot, q: CallbackQuery| {
                let state_storage = state_storage.clone();
                async move {
                    callback_handler(bot, q, state_storage).await
                }
            }
        }))
        .branch(Update::filter_inline_query().endpoint(inline_query_handler));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Show help message")]
    Help,
    #[command(description = "About this bot")]
    About,
    #[command(description = "Show bot statistics (admin only)")]
    Stats,
    #[command(description = "Encode text or reply to a message")]
    Encode(String),
    #[command(description = "Encode text or reply to a message")]
    Hide(String),
    #[command(description = "Decode emoji or reply to a message")]
    Decode(String),
    #[command(description = "Decode emoji or reply to a message")]
    Show(String),
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    db: Option<DbClient>,
    admin_ids: Vec<i64>,
) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            if let Some(db) = db {
                start_handler(bot, msg, db).await
            } else {
                bot.send_message(msg.chat.id, "👋 Welcome! (Running without database)").await?;
                Ok(())
            }
        }
        Command::Help => {
            handlers::help_handler(bot, msg).await
        }
        Command::About => {
            about_handler(bot, msg).await
        }
        Command::Stats => {
            if let Some(db) = db {
                stats_handler(bot, msg, db, admin_ids).await
            } else {
                bot.send_message(msg.chat.id, "❌ Stats unavailable (database not connected)").await?;
                Ok(())
            }
        }
        Command::Encode(text) | Command::Hide(text) => {
            handlers::encode_command_handler(bot, msg, text).await
        }
        Command::Decode(text) | Command::Show(text) => {
            handlers::decode_command_handler(bot, msg, text).await
        }
    }
}
