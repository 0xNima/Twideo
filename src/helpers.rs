extern crate lazy_static;

use std::env;
use regex::Regex;
use rand::Rng;
use twitterVideodl::serde_schemes::*;


lazy_static::lazy_static! {
    static ref TWITTER_STATUS_URL: &'static str = "https://api.twitter.com/1.1/statuses/show.json?extended_entities=true&tweet_mode=extended&id=";
    static ref TWITTER_V2_URL: &'static str = "https://api.twitter.com/2/tweets?expansions=author_id&ids=";
    static ref TWITTER_BEARER_TOKEN: String = format!("Bearer {}", env::var("TWITTER_BEARER_TOKEN").unwrap());
    static ref RE : regex::Regex= Regex::new("https://t.co/\\w+\\b").unwrap();
    pub static ref DATABASE_URL: String = env::var("DATABASE_URL").unwrap();
}

pub fn twitt_id(link: &str) -> TwitterID {
    if link.starts_with("https://twitter.com/") {
        if !link.starts_with("https://twitter.com/i/spaces/") {
            let parsed: Vec<&str> = (&link[20..]).split("/").collect();
            let last_parts: Vec<&str> = parsed.last().unwrap().split("?").collect();            
            let possible_id = last_parts.first().unwrap().parse().unwrap_or(0);   
            if possible_id > 0 {
                return TwitterID::id(possible_id);
            }
        }
    }
    TwitterID::None
}

pub struct TWD {
    pub caption: String,
    pub media_urls: Vec<String>,
    pub r#type: String,
    pub mime_type: Option<String>,
    pub thumb: Option<String>,
    pub name: String,
    pub id: u64
}

pub enum TwitterID {
    id(u64),
    None
}

pub fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    rng.gen_range(10000000..99999999).to_string()
}


pub async fn get_twitter_data(tid: u64) -> Result<Option<TWD>, Box<dyn std::error::Error>> {
    log::info!("Send request to twitter");
    let client = reqwest::Client::new();
    let resp = client.get(format!("{}{}", *TWITTER_STATUS_URL, tid))
                     .header("AUTHORIZATION", &*TWITTER_BEARER_TOKEN)
                     .send()
                     .await?;                     
    log::info!("Status {}", resp.status().as_u16());

    let body = resp.json::<Body>().await?;

    let mut urls: Vec<String> = Vec::new();
    let mut media_type = String::new();
    let mut mime_type: Option<String> = None;
    let mut thumb: Option<String> = None;

    if let Some(extenden_entities) = &body.extended_entities {
        for media in &extenden_entities.media {
            if media_type.is_empty() {
                media_type = media.r#type.clone();
            }

            if thumb.is_none() {
                thumb = media.media_url_https.to_owned();
            }

            if media.r#type == "video" || media.r#type == "animated_gif" {
                let mut last_bitrate = 0;
                let mut last_url = String::new();
                for variant in &media.video_info.as_ref().unwrap().variants {
                    if let Some(bitrate) = variant.bitrate {
                        if bitrate >= last_bitrate {
                            last_url = variant.url.clone();
                            last_bitrate = bitrate;
                        }
                    }
                    if mime_type.is_none() {
                        mime_type = Some(variant.content_type.to_owned());
                    }
                }
                if !last_url.is_empty() {
                    urls.push(last_url);
                }
            } else if media.r#type == "photo" {
                urls.push(media.media_url_https.as_ref().unwrap().to_string());
            }
        }            
    }

    let mut clean_caption = None;

    let captures: Vec<&str> = RE.captures_iter(body.full_text.as_ref().unwrap())
    .map(|c| c.get(0).unwrap().as_str())
    .collect();

    if captures.len() > 0 {
        let mut captured = captures[captures.len() - 1];
        
        // means tweet doesn's contain media, so the link is real link (not media link)
        if urls.is_empty() {
            clean_caption = Some(
                body.full_text.as_ref().unwrap().replace(captured, &format!("\n{}", captured))
            );
        } else {
            clean_caption = Some(
                body.full_text.as_ref().unwrap().replace(captured, "")
            ); // remove media link
            if captures.len() > 1 {
                captured = captures[captures.len() - 2];
                clean_caption = Some(
                    clean_caption.as_ref().unwrap().replace(captured, &format!("\n{}", captured))
                );
            }
        }
    }

    Ok(
        Some(
            TWD {
                caption: format!(
                    "{} \n\n<a href='https://twitter.com/{}'>&#x1F464 {}</a>", 
                    || -> &str {
                        if clean_caption.is_none() {
                            return body.full_text.as_ref().unwrap()
                        }
                        return clean_caption.as_ref().unwrap()
                    }(), 
                    body.user.screen_name, 
                    body.user.name
                ), 
                media_urls: urls,
                r#type: media_type,
                mime_type: mime_type,
                thumb: thumb,
                name: body.user.name,
                id: tid
            }
        )
    )
}