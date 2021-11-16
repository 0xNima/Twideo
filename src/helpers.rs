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
    video_info: VideoInfo,
    r#type: String
}

#[derive(Deserialize, Debug)]
struct ExtendenEntities {
    media: Vec<Media>
}

#[derive(Deserialize, Debug)]
struct Body {
    extended_entities: ExtendenEntities
}


#[derive(Deserialize, Debug)]
struct V2Data {
    author_id: String,
    id: String,
    text: String
}

#[derive(Deserialize, Debug)]
struct User {
    id: String,
    name: String,
    username: String
}

#[derive(Deserialize, Debug)]
struct Includes {
    users: Vec<User>
}

#[derive(Deserialize, Debug)]
struct V2Body {
    data: Vec<V2Data>,
    includes: Includes
}

pub async fn get_video_url(tid: i64) -> Result<Option<String>, Box<dyn std::error::Error>> {
    log::info!("Send request to twitter");
    let client = reqwest::Client::new();
    let resp = client.get(format!("{}{}", *TWITTER_STATUS_URL, tid))
                     .header("AUTHORIZATION", format!("Bearer {}", env::var("TWITTER_BEARER_TOKEN").unwrap()))
                     .send()
                     .await?;                     
    log::info!("Status {}", resp.status().as_u16());

    let body = resp.json::<Body>().await?;
    
    if body.extended_entities.media.len() > 0 {
        for media in &body.extended_entities.media {
            if media.r#type == "video" {
                let mut last_bitrate = 0;
                let mut last_url = String::new();
                for variant in &media.video_info.variants {
                    if let Some(bitrate) = variant.bitrate {
                        if bitrate > last_bitrate {
                            last_url = variant.url.clone();
                            last_bitrate = bitrate;
                        }
                    }
                }
                if !last_url.is_empty() {
                    return Ok(Some(last_url))
                }
            }
        }
    }
    Ok(None)
}

pub async fn get_tweet_data(tid: i64) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client.get(format!("{}{}", *TWITTER_V2_URL, tid))
                     .header("AUTHORIZATION", format!("Bearer {}", env::var("TWITTER_BEARER_TOKEN").unwrap()))
                     .send()
                     .await?;
    log::info!("V2 Status {}", resp.status().as_u16());

    let body = resp.json::<V2Body>().await?;

    let text = &body.data[0].text;
    let user = &body.includes.users[0];
    let name = &user.name;
    let username = &user.username;
    
    let clean_text = RE.replace_all(text, "");
    let caption = format!("{} \n\n<a href='https://twitter.com/{}'>&#x1F464 {}</a>", clean_text, username, name);
    
    Ok(caption)
}