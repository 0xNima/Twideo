extern crate lazy_static;

use std::{env, fmt::format};
use regex::Regex;
use rand::Rng;
use rand::seq::SliceRandom;
use twitterVideodl::serde_schemes::*;


lazy_static::lazy_static! {
    static ref TWITTER_STATUS_URL: &'static str = "https://api.twitter.com/1.1/statuses/show.json?extended_entities=true&tweet_mode=extended&id=";
    static ref TWITTER_V2_URL: &'static str = "https://api.twitter.com/2/tweets?expansions=author_id&ids=";

    static ref TWITTER_BEARER_TOKENS: Vec<String> = vec![
        env::var("TWITTER_BEARER_TOKEN").unwrap_or("".to_string()), 
        env::var("TWITTER_BEARER_TOKEN2").unwrap_or("".to_string())
    ].into_iter().filter(|x| !x.is_empty()).collect::<Vec<String>>();
    
    static ref TWITTER_MULTIMEDIA_URL: &'static str = "https://api.twitter.com/2/tweets";
    static ref TWITTER_EXPANSIONS_PARAMS: &'static str = "expansions=attachments.media_keys,author_id&media.fields=url,variants,preview_image_url&user.fields=name";
    static ref RE : regex::Regex= Regex::new("https://t.co/\\w+\\b").unwrap();
    pub static ref DATABASE_URL: String = env::var("DATABASE_URL").unwrap_or("".to_string());
}

pub fn twitt_id(link: &str) -> TwitterID {
    if link.starts_with("https://twitter.com/") ||  link.starts_with("https://mobile.twitter.com/") {
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

#[derive(Debug)]
pub struct TwitterMedia {
    pub url: String,
    pub r#type: String,
    pub thumb: String
}

#[derive(Debug)]
pub struct TWD {
    pub caption: String,
    pub twitter_media: Vec<TwitterMedia>,
    pub name: String,
    pub id: u64,
    pub extra_urls: Vec<Variant>
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
    
    let token = TWITTER_BEARER_TOKENS.choose(&mut rand::thread_rng()).unwrap().to_string();

    let client = reqwest::Client::new();
    
    let multimedia_response = client.get(
        format!(
            "{}/{}?{}",
            &*TWITTER_MULTIMEDIA_URL,
            tid,
            &*TWITTER_EXPANSIONS_PARAMS
        )
    )
    .header("AUTHORIZATION", format!("Bearer {}", token))
    .send()
    .await?;

    log::info!("Status {}", multimedia_response.status().as_u16());
    
    let multimedia = multimedia_response.json::<MultimediaBody>().await?;

    let mut twitter_media: Vec<TwitterMedia> = Vec::new();
    let mut extra_urls: Vec<Variant> = Vec::new();
    let mut name = String::new();
    let mut username = String::new();


    if let Some(includes) = &multimedia.includes {
        name = includes.users[0].name.to_string();
        username = includes.users[0].username.to_string();

        if let Some(media_set) = &includes.media {
            for media in media_set {
                if media.r#type == "video" || media.r#type == "animated_gif" {
                    let mut last_bitrate = 0;
                    let mut last_url = "";
                    let mut alternative_url = "";
    
                    for variant in media.variants.as_ref().unwrap() {
                        if let Some(bitrate) = variant.bit_rate {
                            extra_urls.push(variant.clone());
                            if bitrate >= last_bitrate {
                                last_url = variant.url.as_str();
                                last_bitrate = bitrate;
                            }
                        } else {
                            alternative_url = variant.url.as_str();
                        }
                    }
    
                    if !last_url.is_empty() {
                        twitter_media.push(TwitterMedia{
                            url: last_url.to_string(),
                            r#type: media.r#type.to_string(),
                            thumb: media.preview_image_url.as_ref().unwrap().to_owned()
                        });
                    } else if !alternative_url.is_empty() {
                        twitter_media.push(TwitterMedia{
                            url: alternative_url.to_string(),
                            r#type: media.r#type.to_string(),
                            thumb: media.preview_image_url.as_ref().unwrap().to_owned()
                        });
                    }
                } else if media.r#type == "photo" {
                    let _url = media.url.as_ref().unwrap().to_string();
                    twitter_media.push(TwitterMedia{
                        url: _url.to_string(),
                        r#type: media.r#type.to_string(),
                        thumb: _url
                    });
                }
            }
        }            
    }

    let mut clean_caption = None;
    let tweet_text = multimedia.data.text.as_ref().unwrap();

    let captures: Vec<&str> = RE.captures_iter(tweet_text)
    .map(|c| c.get(0).unwrap().as_str())
    .collect();

    if captures.len() > 0 {
        let mut captured = captures[captures.len() - 1];
        
        // means tweet doesn's contain media, so the link is real link (not media link)
        if twitter_media.is_empty() {
            clean_caption = Some(
                tweet_text.replace(captured, &format!("\n{}", captured))
            );
        } else {
            clean_caption = Some(
                tweet_text.replace(captured, "")
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
                    "{} \n\n<a href='https://twitter.com/{}/status/{}'>&#x1F464 {}</a>", 
                    || -> &str {
                        if clean_caption.is_none() {
                            return tweet_text
                        }
                        return clean_caption.as_ref().unwrap()
                    }(),
                    username,
                    tid, 
                    name
                ),
                twitter_media,
                name: name,
                id: tid,
                extra_urls: extra_urls
            }
        )
    )
}