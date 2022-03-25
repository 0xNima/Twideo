extern crate lazy_static;
extern crate m3u8_rs;

use reqwest::Client;
use std::env;
use regex::Regex;
use rand::Rng;
use m3u8_rs::playlist::Playlist;
use twitterVideodl::serde_schemes::*;


lazy_static::lazy_static! {
    static ref TWITTER_STATUS_URL: &'static str = "https://api.twitter.com/1.1/statuses/show.json?extended_entities=true&tweet_mode=extended&id=";
    static ref TWITTER_V2_URL: &'static str = "https://api.twitter.com/2/tweets?expansions=author_id&ids=";
    static ref TWITTER_GUEST_TOKEN_URL: &'static str = "https://api.twitter.com/1.1/guest/activate.json";
    static ref TWITTER_BEARER_TOKEN: String = format!("Bearer {}", env::var("TWITTER_BEARER_TOKEN").unwrap());
    static ref TWITTER_GUEST_BEARER_TOKEN: String = format!("Bearer {}", env::var("TWITTER_GUEST_BEARER_TOKEN").unwrap());
    static ref TWITTER_AUDIO_SPACE_URL: String = format!(
        "https://twitter.com/i/api/graphql/{}/AudioSpaceById?variables=\
        %7B%22id%22%3A%22{{ID}}%22%2C%22isMetatagsQuery%22%3Afalse%2C%22withSuperFollowsUserFields%22%3Afalse%2C\
        %22withDownvotePerspective%22%3Afalse%2C%22withReactionsMetadata%22%3Afalse%2C%22withReactionsPerspective%22%3Afalse%2C\
        %20%22withSuperFollowsTweetFields%22%3Afalse%2C%22withReplays%22%3Afalse%2C\
        %22__fs_dont_mention_me_view_api_enabled%22%3Afalse%2C%22__fs_interactive_text_enabled%22%3Afalse%2C\
        %22__fs_responsive_web_uc_gql_enabled%22%3Afalse%7D
        ",
        env::var("GRAPHQL_PATH").unwrap()
    );
    static ref TWITTER_SPACE_METADATA_URL: &'static str = "https://twitter.com/i/api/1.1/live_video_stream/status/{MEDIA_KEY}?client=web&use_syndication_guest_id=false&cookie_set_host=twitter.com";
    static ref RE : regex::Regex= Regex::new("https://t.co/\\w+\\b").unwrap();
    pub static ref DATABASE_URL: String = env::var("DATABASE_URL").unwrap();
}

pub fn twitt_id(link: &str) -> TwitterID {
    if link.starts_with("https://twitter.com/") {
        if (&link[20..29]).starts_with("i/spaces/") {
            let splited: Vec<&str> = (&link[29..]).split("?").collect();
            if splited.len() > 0 {
                return TwitterID::space_id(splited[0].to_string())
            }
        } else {
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
    space_id(String),
    None
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

pub fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    rng.gen_range(10000000..99999999).to_string()
}

async fn guest_token(client: &Client) -> Option<String> {
    let resp = client.post(*TWITTER_GUEST_TOKEN_URL)
                     .header("AUTHORIZATION", &*TWITTER_GUEST_BEARER_TOKEN)
                     .send()
                     .await;
    if let Ok(response) = resp {
        if response.status().as_u16() != 200 {
            return None;
        }

        if let Ok(token) = response.json::<GuestToken>().await {
            return  Some(token.guest_token);
        }
    }
    return None;
}

fn media_key_url(id: &str) -> String {
    return TWITTER_AUDIO_SPACE_URL.replace("{ID}", id)
}

fn metadata_url(media_key: &str) -> String {
    return TWITTER_SPACE_METADATA_URL.replace("{MEDIA_KEY}", media_key)
}

async fn space_media_key(client: &Client, space_id: &str) -> Option<SpaceMetadata> {
    if let Some(token) = guest_token(&client).await {
        let resp = client.get(&media_key_url(space_id))
                     .header("x-guest-token", &token)
                     .header("Authorization", &*TWITTER_GUEST_BEARER_TOKEN)
                     .send()
                     .await;
        
        if let Ok(response) = resp {
            if response.status().as_u16() != 200 {
                return None;
            }
            if let Ok(mut obj) = response.json::<SpaceObject>().await {
                obj.data.audioSpace.metadata.token = Some(token);
                return Some(obj.data.audioSpace.metadata);
            }
        }
    }
    return None
}

async fn space_playlist(client: &Client, space_id: &str) -> Option<String> {
    if let Some(space_obj) = space_media_key(client, space_id).await {
        let resp = client.get(metadata_url(&space_obj.media_key))
                     .header("AUTHORIZATION", &*TWITTER_GUEST_BEARER_TOKEN)
                     .header("X-Guest-Token", space_obj.token.unwrap())
                     .send()
                     .await;
        if let Ok(response) = resp {
            if response.status().as_u16() != 200 {
                return None;
            }
            let data = response.json::<SpacePlaylist>().await.unwrap();
            return Some(data.source.location)
        }
    }
    return None
}

async fn download_space(client: &Client, location: &str) {
    let resp = client.get(location)
    .send()
    .await;

    if let Ok(response) = resp {
        if response.status().as_u16() != 200 {
            return;
        }
        
        let bytes = response.bytes().await.unwrap();

        match m3u8_rs::parse_playlist_res(&bytes) {
            Ok(Playlist::MasterPlaylist(pl)) => println!("Master playlist:\n{:?}", pl),
            Ok(Playlist::MediaPlaylist(pl)) => println!("Media playlist:\n{:?}", pl),
            Err(e) => println!("Error: {:?}", e)
        }
    
    }
}

pub async fn get_space(space_id: &str) {
    let client = reqwest::Client::new();
    if let Some(playlist) = space_playlist(&client, space_id).await {
        download_space(&client, &playlist).await;
    }
}