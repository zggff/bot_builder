#![feature(once_cell)]

use std::{error::Error, lazy::SyncLazy};

use dptree::endpoint;
use serde::{Deserialize, Serialize};
use teloxide::{dispatching2::UpdateFilterExt, prelude2::*, utils::command::BotCommand};
use teloxide::{
    payloads::SendMessageSetters,
    prelude2::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    RequestError,
};

use crate::catalogue::{Catalogue, Index};

mod catalogue;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "start the bot")]
    Start,
    #[command(description = "display this text.")]
    Help,
    #[command(description = "buy goods")]
    Buy,
}

#[derive(Debug, Serialize, Deserialize)]
struct Product {
    price: usize,
    title: String,
    description: String,
}
static PRODUCTS: SyncLazy<Catalogue<Product, String>> = SyncLazy::new(|| {
    let file_path = std::env::var("INPUT_FILE").expect("input file path is not specified");
    let json = std::fs::read_to_string(file_path).unwrap();
    serde_json::from_str(&json).unwrap()
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv()?;
    log4rs::init_file("log.yaml", Default::default()).unwrap();
    run().await
}

async fn run() -> Result<(), Box<dyn Error>> {
    log::info!("Starting bot...");

    let bot = Bot::from_env().auto_send();
    let message_handler = Update::filter_message().branch(endpoint(message_handler));
    let callback_handler = Update::filter_callback_query().branch(endpoint(callback_handler));
    let handler = dptree::entry()
        .branch(message_handler)
        .branch(callback_handler);
    Dispatcher::builder(bot, handler)
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
    Ok(())
}

fn generate_keyboard(index: &Index) -> Option<InlineKeyboardMarkup> {
    let entries = PRODUCTS.get(index)?;
    if let Catalogue::List { list, .. } = entries {
        let keyboard: Vec<Vec<InlineKeyboardButton>> = list
            .chunks(3)
            .enumerate()
            .map(|(local_index, entries)| {
                entries
                    .iter()
                    .map(|entrie| match entrie {
                        Catalogue::List { data, .. } => InlineKeyboardButton::callback(
                            data.to_owned(),
                            index.join(local_index).to_string(),
                        ),
                        Catalogue::Item(item) => InlineKeyboardButton::callback(
                            item.title.to_owned(),
                            index.join(local_index).to_string(),
                        ),
                    })
                    .collect()
            })
            .collect();
        Some(InlineKeyboardMarkup::new(keyboard))
    } else {
        None
    }
}

pub async fn callback_handler(
    bot: AutoSend<Bot>,
    callback: CallbackQuery,
) -> Result<(), RequestError> {
    let data = callback.data.as_ref().unwrap();
    let index = data.parse::<Index>().unwrap();
    let products = PRODUCTS.get(&index).unwrap();
    match products {
        Catalogue::List { data, list } => {
            let keyboard = generate_keyboard(&index).unwrap();
            bot.send_message(callback.id, "buy")
                .reply_markup(keyboard)
                .await?;
        }
        Catalogue::Item(item) => {}
    }
    Ok(())
}
pub async fn message_handler(bot: AutoSend<Bot>, message: Message) -> Result<(), RequestError> {
    let text = message.text();
    if text.is_none() {
        return respond(());
    }
    if let Ok(command) = Command::parse(text.unwrap(), "zggff bot") {
        match command {
            Command::Start => bot.send_message(message.chat.id, "this is the bot").await?,
            Command::Help => {
                bot.send_message(message.chat.id, Command::descriptions())
                    .await?
            }
            Command::Buy => {
                bot.send_message(message.chat.id, "buy")
                    .reply_markup(InlineKeyboardMarkup::new([[
                        InlineKeyboardButton::callback("buy".to_string(), "/0/".to_string()),
                    ]]))
                    .await?
            }
        };
    }
    Ok(())
}
