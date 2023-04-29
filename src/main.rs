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
        InlineKeyboardMarkup, InlineQueryResultGif, ChatId
    }, 
    payloads::SendMessageSetters,     
};
use helpers::{get_twitter_data, twitt_id, DATABASE_URL, generate_code, get_thread, TWD, TwitterID};
use twitterVideodl::{DBManager, serde_schemes::Variant};
use reqwest::Url;


struct MediaWithExtra {
    media: Vec<InputMedia>,
    extra_urls: Vec<Variant>,
    caption: String,
    allowed: bool,
    keyboard: Option<Vec<Vec<InlineKeyboardButton>>>
}

struct TextResponse {
    text: String,
    keyboard: Option<Vec<Vec<InlineKeyboardButton>>>
}

enum Response {
    Media(MediaWithExtra),
    Text(TextResponse),
    InlineResults(Vec<InlineQueryResult>),
    Unauthorized(i32),
    TooManyRequest(i32),
    None
}

const FULL_ALBUM: u8 = 1;
const THREAD: u8 = 2;

fn message_response_cb(twitter_data: &TWD) -> Response {
    let mut caption_is_set = false;
    let mut media_group = Vec::new();
    let mut allowed = false;

    let mut keyboard;

    if twitter_data.thread_count > 0 && twitter_data.next <= twitter_data.thread_count as u8 {
        keyboard = Some(
            vec![
                vec![
                    InlineKeyboardButton::callback(
                        "Next thread".to_string(),
                        format!(
                            "{}_{}_{}_{}",
                            THREAD,
                            twitter_data.conversation_id,
                            twitter_data.user_id,
                            twitter_data.next
                        )
                    )
                ]
            ]
        );
    } else {
        keyboard = None;
    }

    for media in &twitter_data.twitter_media {
        let input_file = InputFile::url(
            Url::parse(
                &media.url
            ).unwrap()
        );

        if media.r#type == "photo" {
            let mut tl_media = InputMediaPhoto::new(input_file);
            if !caption_is_set {
                tl_media = tl_media.caption(&twitter_data.caption)
                .parse_mode(ParseMode::Html);
                caption_is_set = true;
            }
            media_group.push(InputMedia::Photo(tl_media));
        } else if media.r#type == "video" || media.r#type == "animated_gif" {
            allowed = true;
            let mut tl_media = InputMediaVideo::new(input_file);
            if !caption_is_set {
                tl_media = tl_media.caption(&twitter_data.caption)
                .parse_mode(ParseMode::Html);
                caption_is_set = true;
            }
            media_group.push(InputMedia::Video(tl_media));
        }
    }
    if !caption_is_set {
        return Response::Text(
            TextResponse { 
                text: twitter_data.caption.to_string(), 
                keyboard 
            }
        );
    }

    return Response::Media(
        MediaWithExtra{
            media: media_group, 
            extra_urls: twitter_data.extra_urls.to_vec(),
            caption: twitter_data.caption.to_string(),
            allowed,
            keyboard
        }
    );
}


fn inline_query_response_cb(twitter_data: &TWD) -> Response {
    let mut inline_result: Vec<InlineQueryResult> = Vec::new();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    if twitter_data.thread_count > 0 {
        keyboard.push(vec![
            InlineKeyboardButton::callback(
                "Next thread".to_string(), 
                format!("{}_{}_{}_{}", THREAD, twitter_data.conversation_id, twitter_data.user_id, twitter_data.next)
            )
        ]);
    }

    if twitter_data.twitter_media.len() > 1 {
        keyboard.push(vec![
            InlineKeyboardButton::callback(
                "See album".to_string(),
                format!("{}_{}", FULL_ALBUM, &twitter_data.id)
            )   
        ]);
    }

    if twitter_data.twitter_media.is_empty() {
        let mut article_result = InlineQueryResultArticle::new(
            generate_code(),
            &twitter_data.name,
            InputMessageContent::Text(
                InputMessageContentText::new(&twitter_data.caption)
                    .parse_mode(ParseMode::Html)
                    .disable_web_page_preview(true),
            ),
        )
        .description(&twitter_data.caption);

        if keyboard.len() > 0 {
            article_result = article_result.reply_markup(InlineKeyboardMarkup::new(keyboard));
        }

        inline_result.push(InlineQueryResult::Article(article_result));

        return Response::InlineResults(inline_result);
    }

    for media in &twitter_data.twitter_media {
        match media.r#type.as_str() {
            "photo" => {
                let mut inline_photo = InlineQueryResultPhoto::new(
                    generate_code(), 
                    Url::parse(&media.url).unwrap(),
                    Url::parse(&media.thumb).unwrap()
                )
                .title(&twitter_data.name)
                .caption(&twitter_data.caption)
                .parse_mode(ParseMode::Html);

                if keyboard.len() > 0 {
                    inline_photo = inline_photo.reply_markup(
                        InlineKeyboardMarkup::new(keyboard.clone())
                    );
                }

                inline_result.push(InlineQueryResult::Photo(inline_photo));
            },
            "video" => {
                for variant in &twitter_data.extra_urls {
                    let mut inline_video = InlineQueryResultVideo::new(
                        generate_code(), 
                        Url::parse(variant.url.as_str()).unwrap(),
                        variant.content_type.parse().unwrap(),
                        Url::parse(&media.thumb).unwrap(), 
                        format!("{} (Bitrate {})", &twitter_data.name, variant.bit_rate.unwrap_or(0))
                    )
                    .caption(&twitter_data.caption)
                    .parse_mode(ParseMode::Html);

                    if keyboard.len() > 0 {
                        inline_video = inline_video.reply_markup(
                            InlineKeyboardMarkup::new(keyboard.clone())
                        );
                    }
    
                    inline_result.push(InlineQueryResult::Video(inline_video));
                }
            },
            "animated_gif" => {
                for variant in &twitter_data.extra_urls {
                    let mut inline_gifs = InlineQueryResultGif::new(
                        generate_code(), 
                        Url::parse(variant.url.as_str()).unwrap(),
                        Url::parse(&media.thumb).unwrap(),

                    )
                    .caption(&twitter_data.caption)
                    .parse_mode(ParseMode::Html)
                    .title(format!("{} (Bitrate {})", twitter_data.name, variant.bit_rate.unwrap_or(0)));

                    if keyboard.len() > 0 {
                        inline_gifs = inline_gifs.reply_markup(
                            InlineKeyboardMarkup::new(keyboard.clone())
                        );
                    }
    
                    inline_result.push(InlineQueryResult::Gif(inline_gifs));
                }
            },
            _ => {}
        }
    }
    return Response::InlineResults(inline_result);
}

async fn convert_to_tl<F>(url: &str, callback: F) -> Response where
    F: Fn(&TWD) -> Response {
        match twitt_id(url) {
            TwitterID::id(id) => {
                let data = get_twitter_data(id).await;

                if data.is_ok() {
                    if let Some(twitter_data) = data.unwrap() {
                        return callback(&twitter_data);
                    }
    
                    return Response::TooManyRequest(429);
                }
                return Response::Unauthorized(401);
            },
            _ => {}
        }
        return Response::None
}

async fn convert_to_tl_by_id<F>(id: u64, next: u8, callback: F) -> Response where
    F: Fn(&TWD) -> Response {
    let data = get_twitter_data(id).await;

    if data.is_ok() {
        if let Some(mut twitter_data) = data.unwrap() {
            twitter_data.next = next;
            return callback(&twitter_data);
        }

        return Response::TooManyRequest(429)
    }

    return Response::Unauthorized(401)
}

async fn response_matching(r: Response, bot: &AutoSend<Bot> , chat_id: i64) 
-> Result<(), Box<dyn Error + Send + Sync>> {
    match r {
        Response::Text(response) => {
            let msg = bot.send_message(chat_id, response.text)
            .parse_mode(ParseMode::Html)
            .disable_web_page_preview(true);

            if let Some(keyboard) = response.keyboard {
                msg.reply_markup(
                    InlineKeyboardMarkup::new(keyboard)
                )
                .await?;
            } else {
                msg.await?;
            }
        },
        Response::Media(media_with_extra) => {
            let response = bot.send_media_group(
                chat_id, 
                media_with_extra.media
            )
            .await;

            if response.is_ok() {
                if let Some(keyboard) = media_with_extra.keyboard {
                    bot.send_message(chat_id, "tap button to see next thread")
                    .parse_mode(ParseMode::Html)
                    .disable_web_page_preview(true)
                    .reply_markup(
                        InlineKeyboardMarkup::new(keyboard)
                    )
                    .await?;   
                }
            } else if media_with_extra.allowed {
                bot.send_message(
                    chat_id,
                    format!("Telegram is unable to download high quality video.\nI will send you other qualities.")
                ).parse_mode(ParseMode::Html)
                .disable_web_page_preview(true)
                .await?;

                for variant in &media_with_extra.extra_urls {
                    bot.send_media_group(chat_id, [
                        InputMedia::Video(
                            InputMediaVideo::new(
                                InputFile::url(Url::parse(variant.url.as_str()).unwrap())
                            )
                            .caption(&media_with_extra.caption)
                            .parse_mode(ParseMode::Html)
                        )
                    ])
                    .await?;
                }
            }
        },
        Response::TooManyRequest(code) => {
            bot.send_message(chat_id, "🧑‍💻👨‍💻⚠️ Server is busy! Please try a little later.")
            .disable_web_page_preview(true)
            .await?;
        },
        Response::Unauthorized(code) => {
            bot.send_message(chat_id, "☠️ Bot is stopped to work due to Twitter's new API plan(<a href='https://twitter.com/TwitterDev/status/1641222786894135296'>click to see announcement</a>). But don't despair. 👀 I'm looking for a way to come back. Be patient 💪🏻")
            .parse_mode(ParseMode::Html)
            .disable_web_page_preview(true)
            .await?;
        },
        _ => ()
    }

    Ok(())
}

async fn message_handler(
    m: Message,
    bot: AutoSend<Bot>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat = &m.chat;
    let username = chat.username().map(String::from);
    let dbm = DBManager::new(&&DATABASE_URL);

    if dbm.is_ok() {
        dbm.unwrap().create_user(
            chat.id, 
            format!("{} {}", chat.first_name().unwrap_or(""), chat.last_name().unwrap_or("")),
            username
        );
    }

    if let Some(maybe_url) = m.text() {
        if maybe_url == "/start" {
            bot.send_message(chat.id, "👉  Send me a valid twitter url").await?;
        }
        else {
            let response = convert_to_tl(maybe_url, message_response_cb).await;
            response_matching(response, &bot, chat.id).await?;
        }
    }

    Ok(())
}

async fn inline_queries_handler(
    bot: AutoSend<Bot>,
    update: InlineQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let response = convert_to_tl(&update.query, inline_query_response_cb).await;

    match response {
        Response::InlineResults(inline_result) => {
            let req_builder = bot.answer_inline_query(update.id, inline_result);
            req_builder.await?;
        },
        Response::Unauthorized(code) => {
            bot.answer_inline_query(
                update.id, 
                [
                    InlineQueryResult::Article(
                        InlineQueryResultArticle::new(
                            generate_code(),
                            "Stop Working",
                            InputMessageContent::Text(
                                InputMessageContentText::new("☠️ Bot is stopped to work due to Twitter's new API plan(<a href='https://twitter.com/TwitterDev/status/1641222786894135296'>click to see announcement</a>). But don't despair. 👀 I'm looking for a way to come back. Be patient 💪🏻")
                                    .parse_mode(ParseMode::Html)
                                    .disable_web_page_preview(true),
                            ),
                        )
                    )
                ]
            ).await?;
        },
        Response::TooManyRequest(code) => {
            bot.answer_inline_query(
                update.id, 
                [
                    InlineQueryResult::Article(
                        InlineQueryResultArticle::new(
                            generate_code(),
                            "Busy",
                            InputMessageContent::Text(
                                InputMessageContentText::new("🧑‍💻👨‍💻⚠️ Server is busy! Please try a little later.")
                                    .disable_web_page_preview(true),
                            ),
                        )
                    )
                ]
            ).await?;
        },
        _ => ()
    }

    Ok(())
}

async fn callback_queries_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let query = q.data.unwrap();
    let query_parts = query.split('_').collect::<Vec<&str>>();
    
    if query_parts.len() < 2 || query_parts.len() > 4 {
        // for backward compatibility. Maybe some old reply markups contain invalid data format
        return Ok(());
    }
    let query_type = query_parts[0].parse::<u8>().unwrap();

    match query_type {
        FULL_ALBUM => {
            // query template: <query-type>_<tweet-id>
            let tid = query_parts[1].parse::<u64>().unwrap();
            let response = convert_to_tl_by_id(tid, 1, message_response_cb).await;
            response_matching(response, &bot, q.from.id).await?;
        },
        THREAD => {
            // query template: <query-type>_<conversation-id>_<user-id>_<thread-number>
            let conversation_id = query_parts[1].parse::<u64>().unwrap();
            let user_id = query_parts[2].parse::<u64>().unwrap();
            let thread_number = query_parts[3].parse::<u8>().unwrap();

            let tid = get_thread(conversation_id, thread_number, user_id).await;

            if let Some(tweet_id) = tid {
                let response = convert_to_tl_by_id(
                    tweet_id,
                    thread_number + 1,
                    message_response_cb
                ).await;
                response_matching(response, &bot, q.from.id).await?;   
            } else {
                bot.send_message(q.from.id, "Thread not found 🤷‍♂️").await?;
            }
            
        },
        _ => {}
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
