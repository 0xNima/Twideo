extern crate lazy_static;

use serde::Deserialize;
use std::env;
use regex::Regex;


lazy_static::lazy_static! {
    static ref TWITTER_STATUS_URL: &'static str = "https://api.twitter.com/1.1/statuses/show.json?extended_entities=true&tweet_mode=extended&id=";
    static ref TWITTER_V2_URL: &'static str = "https://api.twitter.com/2/tweets?expansions=author_id&ids=";
    static ref RE : regex::Regex= Regex::new("https://t.co/\\w+\\b").unwrap();
}

pub fn twitt_id(link: &str) -> Option<i64>{
    let mut possible_id: i64 = 0;
    if let Some(_) = link.find("twitter.com") {
        let parsed: Vec<&str> = link.split("/").collect();
        let last_parts: Vec<&str> = parsed.last().unwrap().split("?").collect();
        possible_id = last_parts.first().unwrap().parse().unwrap_or(0);
    }
    if possible_id > 0 {
        return Some(possible_id);
    }
    None
}


#[derive(Deserialize, Debug)]
struct Variant {
    bitrate: Option<i32>,
    content_type: String,
    url: String
}
 
#[derive(Deserialize, Debug)]
struct VideoInfo {
    variants: Vec<Variant>
}

#[derive(Deserialize, Debug)]
struct Media {
    video_info: Option<VideoInfo>,
    r#type: String,
    media_url_https: Option<String>
}

#[derive(Deserialize, Debug)]
struct ExtendenEntities {
    media: Vec<Media>
}

#[derive(Deserialize, Debug)]
struct User {
    id_str: String,
    name: String,
    screen_name: String
}

#[derive(Deserialize, Debug)]
struct Body {
    extended_entities: Option<ExtendenEntities>,
    full_text: Option<String>,
    user: User
}


pub struct TWD {
    pub caption: String,
    pub media_urls: Vec<String>,
    pub r#type: String
}

pub async fn get_twitter_data(tid: i64) -> Result<Option<TWD>, Box<dyn std::error::Error>> {
    log::info!("Send request to twitter");
    let client = reqwest::Client::new();
    let resp = client.get(format!("{}{}", *TWITTER_STATUS_URL, tid))
                     .header("AUTHORIZATION", format!("Bearer {}", env::var("TWITTER_BEARER_TOKEN").unwrap()))
                     .send()
                     .await?;                     
    log::info!("Status {}", resp.status().as_u16());

    let body = resp.json::<Body>().await?;

    let mut urls: Vec<String> = Vec::new();
    let mut media_type = String::new();

    if let Some(extenden_entities) = &body.extended_entities {
        for media in &extenden_entities.media {
            if media_type.is_empty() {
                media_type = media.r#type.clone();
            }
    
            if media.r#type == "video" {
                let mut last_bitrate = 0;
                let mut last_url = String::new();
                for variant in &media.video_info.as_ref().unwrap().variants {
                    if let Some(bitrate) = variant.bitrate {
                        if bitrate > last_bitrate {
                            last_url = variant.url.clone();
                            last_bitrate = bitrate;
                        }
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
   
    let clean_caption = body.full_text.as_ref().map(|text|{ return RE.replace_all(text, "") }).unwrap();
    
    Ok(
        Some(
            TWD {
                caption: format!(
                    "{} \n\n<a href='https://twitter.com/{}'>&#x1F464 {}</a>", 
                    clean_caption, 
                    body.user.screen_name, 
                    body.user.name
                ), 
                media_urls: urls,
                r#type: media_type
            }
        )
    )
}