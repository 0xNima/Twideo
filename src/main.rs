extern crate dotenv;
extern crate twitterVideodl;

mod helpers;

use teloxide::prelude::*;
use dotenv::dotenv;
use std::env;
use teloxide::types::{InputFile, InputMedia, InputMediaVideo, ParseMode};
use helpers::{get_video_url, twitt_id, get_tweet_data};
use twitterVideodl::{DBManager};

#[tokio::main]
async fn main() {
    dotenv().ok();
    teloxide::enable_logging!();
    log::info!("Starting Twideo");

    let bot = Bot::from_env().auto_send();

    teloxide::repl(bot, |message| async move {
        let chat = &message.update.chat;
        let username = chat.username().map(String::from);
        let dbm = DBManager::new(&env::var("DATABASE_URL").unwrap()).unwrap();

        dbm.create_user(
            chat.id, 
            format!("{} {}", chat.first_name().unwrap(), chat.last_name().unwrap_or("")),
            username
        );
        if let Some(link) = message.update.text() {
            if link == "/start" {
                message.answer_str("👉  Send me a valid twitter url").await?;
            }
            else if let Some(id) = twitt_id(link) {
                let video_url = get_video_url(id).await.unwrap_or(None);
                let caption = get_tweet_data(id).await.unwrap_or("".to_string());
                if let Some(url) = video_url {
                    let media = InputMediaVideo::new(InputFile::url(url))
                                .caption(caption)
                                .parse_mode(ParseMode::Html);
                    let media_group = vec!{
                        InputMedia::Video(media)
                    };
                    message.answer_media_group(media_group).await?;
                }
            }
        }    
        respond(())
    })
    .await;
}
