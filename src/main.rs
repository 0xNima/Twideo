extern crate dotenv;
extern crate twitterVideodl;

mod helpers;

use dotenv::dotenv;
use std::{env, error::Error};
use teloxide::{
    prelude2::*, 
    types::{
        ParseMode, 
        InputFile, 
        InputMedia, 
        InputMediaVideo, 
        InputMediaPhoto,
        InlineQueryResult,
        InlineQueryResultArticle,
        InputMessageContent,
        InputMessageContentText, 
        InlineQueryResultVideo, 
        InlineQueryResultPhoto,
        InlineKeyboardButton,
        InlineKeyboardMarkup, InlineQueryResultGif
    }, 
    payloads::SendMessageSetters,     
};
use helpers::{get_twitter_data, twitt_id, generate_code, DATABASE_URL, TWD, TwitterID, get_space};
use twitterVideodl::{DBManager};
use reqwest::Url;


enum Response {
    Media(Vec<InputMedia>),
    Text(String),
    InlineResults(Vec<InlineQueryResult>),
    None
}

fn message_response_cb(twitter_data: &TWD) -> Response {
    let mut caption_is_set = false;
    let mut media_group = Vec::new();

    for url in &twitter_data.media_urls {

        if &twitter_data.r#type == "photo" {
            let mut media = InputMediaPhoto::new(InputFile::url(Url::parse(url).unwrap()));
            if !caption_is_set {
                media = media.caption(&twitter_data.caption)
                .parse_mode(ParseMode::Html);
                caption_is_set = true;
            }
            media_group.push(InputMedia::Photo(media));
        } else if &twitter_data.r#type == "video" || &twitter_data.r#type == "animated_gif" {
            let mut media = InputMediaVideo::new(InputFile::url(Url::parse(url).unwrap()));
            if !caption_is_set {
                media = media.caption(&twitter_data.caption)
                .parse_mode(ParseMode::Html);
                caption_is_set = true;
            }
            media_group.push(InputMedia::Video(media));
        }
    }
    if !caption_is_set {
        return Response::Text(twitter_data.caption.to_owned());
    }
    return Response::Media(media_group);
}


fn inline_query_response_cb(twitter_data: &TWD) -> Response {
    let mut inline_result: Vec<InlineQueryResult> = Vec::new();

    for url in &twitter_data.media_urls {
        if &twitter_data.r#type == "photo" {
            let mut inline_photo = InlineQueryResultPhoto::new(
                generate_code(), 
                Url::parse(url).unwrap(),
                Url::parse(&twitter_data.thumb.as_ref().unwrap_or(&"".to_owned())).unwrap()
            )
            .title(twitter_data.name.to_owned())
            .caption(twitter_data.caption.to_owned())
            .parse_mode(ParseMode::Html);

            if twitter_data.media_urls.len() > 1 {
                let keyboard: Vec<Vec<InlineKeyboardButton>> = vec![
                    vec![
                        InlineKeyboardButton::callback(
                            "see album".to_string(),
                            twitter_data.id.to_string()
                        )
                    ]
                ];

                inline_photo = inline_photo.reply_markup(
                    InlineKeyboardMarkup::new(keyboard)
                );
            }
            
            inline_result.push(InlineQueryResult::Photo(inline_photo));

            break
        } else if twitter_data.r#type == "video" {
            inline_result.push(InlineQueryResult::Video(
                InlineQueryResultVideo::new(
                    generate_code(), 
                    Url::parse(url).unwrap(),
                    twitter_data.mime_type.as_ref().unwrap().parse().unwrap(),
                    Url::parse(&twitter_data.thumb.as_ref().unwrap_or(&"".to_owned())).unwrap(), 
                    twitter_data.name.to_owned()
                )
                .caption(twitter_data.caption.to_owned())
                .parse_mode(ParseMode::Html)
            ));
        } else if twitter_data.r#type == "animated_gif" {
            inline_result.push(InlineQueryResult::Gif(
                InlineQueryResultGif::new(
                    generate_code(), 
                    Url::parse(url).unwrap(),
                    Url::parse(&twitter_data.thumb.as_ref().unwrap_or(&"".to_owned())).unwrap(), 
                )
                .caption(twitter_data.caption.to_owned())
                .parse_mode(ParseMode::Html)
                .title(twitter_data.name.to_owned())
            ));
        }
    }

    if &twitter_data.r#type != "photo" && &twitter_data.r#type != "video" && &twitter_data.r#type != "animated_gif" {
        inline_result.push(InlineQueryResult::Article(
            InlineQueryResultArticle::new(
                generate_code(),
                twitter_data.name.to_owned(),
                InputMessageContent::Text(
                    InputMessageContentText::new(twitter_data.caption.to_owned())
                        .parse_mode(ParseMode::Html)
                        .disable_web_page_preview(true),
                ),
            )
            .description(twitter_data.caption.to_owned()),
        ));
    }
    return Response::InlineResults(inline_result);
}

async fn convert_to_tl<F>(url: &str, callback: F) -> Response where
    F: Fn(&TWD) -> Response {
        match twitt_id(url) {
            TwitterID::id(id) => {
                let data = get_twitter_data(id).await.unwrap_or(None);
                if let Some(twitter_data) = data {
                    return callback(&twitter_data);
                }
            },
            TwitterID::space_id(space_id) => {
                get_space(&space_id).await;
            },
            _ => {}
        }
        return Response::None
}

async fn convert_to_tl_by_id<F>(id: u64, callback: F) -> Response where
    F: Fn(&TWD) -> Response {
    let data = get_twitter_data(id).await.unwrap_or(None);
    if let Some(twitter_data) = data {
        return callback(&twitter_data);
    }
    return Response::None
}

async fn message_handler(
    m: Message,
    bot: AutoSend<Bot>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat = &m.chat;
    let username = chat.username().map(String::from);
    let dbm = DBManager::new(&&DATABASE_URL).unwrap();

    dbm.create_user(
        chat.id, 
        format!("{} {}", chat.first_name().unwrap_or(""), chat.last_name().unwrap_or("")),
        username
    );

    if let Some(maybe_url) = m.text() {
        if maybe_url == "/start" {
            bot.send_message(chat.id, "👉  Send me a valid twitter url").await?;
        }
        else {
            let response = convert_to_tl(maybe_url, message_response_cb).await;

            match response {
                Response::Text(caption) => {
                    bot.send_message(chat.id, caption)
                    .parse_mode(ParseMode::Html)
                    .disable_web_page_preview(true)
                    .await?;
                },
                Response::Media(media_group) => {
                    bot.send_media_group(chat.id, media_group).await?;
                },
                _ => ()
            }
        }
    }

    Ok(())
}

async fn inline_queries_handler(
    bot: AutoSend<Bot>,
    update: InlineQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let response = convert_to_tl(&update.query, inline_query_response_cb).await;
    if let Response::InlineResults(inline_result) = response {
        let req_builder = bot.answer_inline_query(update.id, inline_result);
        req_builder.await?;
    }

    Ok(())
}

async fn callback_queries_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tid: u64 = q.data.unwrap().parse().unwrap();
    let response = convert_to_tl_by_id(tid, message_response_cb).await;

    match response {
        Response::Media(media_group) => {
            bot.send_media_group(q.from.id, media_group).await?;
        },
        _ => ()
    }
    Ok(())
}


#[tokio::main]
async fn main() {
    dotenv().ok();
    teloxide::enable_logging!();

    log::info!("Starting Twideo");

    let bot = Bot::from_env().auto_send();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_inline_query().endpoint(inline_queries_handler))
        .branch(Update::filter_callback_query().endpoint(callback_queries_handler));

    Dispatcher::builder(bot, handler)
    .default_handler(|_| async {})
    .build()
    .setup_ctrlc_handler().dispatch().await;
}
