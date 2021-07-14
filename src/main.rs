extern crate dotenv;

mod helpers;

use teloxide::prelude::*;
use dotenv::dotenv;
use std::env;
use teloxide::types::InputFile;
use helpers::{get_video_url, twitt_id};

#[tokio::main]
async fn main() {
    dotenv().ok();
    teloxide::enable_logging!();
    log::info!("Starting Twideo");

    let bot = Bot::from_env().auto_send();

    teloxide::repl(bot, |message| async move {
        if let Some(link) = message.update.text() {
            if let Some(id) = twitt_id(link) {
                let video_url = get_video_url(id).await.unwrap_or(None);
                if let Some(url) = video_url {
                    let _bot = message.requester;
                    _bot.send_video(message.update.chat.id, InputFile::url(url)).await?;
                }
            }
        }    
        respond(())
    })
    .await;
}
