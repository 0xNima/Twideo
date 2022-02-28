extern crate dotenv;
extern crate twitterVideodl;

mod helpers;

use teloxide::prelude::*;
use dotenv::dotenv;
use std::env;
use teloxide::types::{
    InputFile, 
    InputMedia, 
    InputMediaVideo, 
    InputMediaPhoto, 
    ParseMode
};
use helpers::{get_twitter_data, twitt_id};
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
            format!("{} {}", chat.first_name().unwrap_or(""), chat.last_name().unwrap_or("")),
            username
        );
        if let Some(link) = message.update.text() {
            if link == "/start" {
                message.answer("👉  Send me a valid twitter url").await?;
            }
            else if let Some(id) = twitt_id(link) {
                let data = get_twitter_data(id).await.unwrap_or(None);
                if let Some(twitter_data) = data {

                    let mut media_group = Vec::new();
                    let mut caption_is_set = false;

                    for url in &twitter_data.media_urls {
                        if &twitter_data.r#type == "photo" {
                            let mut media = InputMediaPhoto::new(InputFile::url(url));
                            if !caption_is_set {
                                media = media.caption(&twitter_data.caption)
                                .parse_mode(ParseMode::Html);
                                caption_is_set = true;
                            }
                            media_group.push(InputMedia::Photo(media));
                        } else if &twitter_data.r#type == "video" {
                            let mut media = InputMediaVideo::new(InputFile::url(url));
                            if !caption_is_set {
                                media = media.caption(&twitter_data.caption)
                                .parse_mode(ParseMode::Html);
                                caption_is_set = true;
                            }
                            media_group.push(InputMedia::Video(media));
                        }
                    }
                    if !caption_is_set {
                        message.answer(&twitter_data.caption)
                        .parse_mode(ParseMode::Html)
                        .disable_web_page_preview(true)
                        .await?;
                    } else {
                        message.answer_media_group(media_group).await?;
                    }
                }
            }
        }    
        respond(())
    })
    .await;
}
