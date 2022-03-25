#[macro_use] extern crate diesel;

pub mod scheme;
pub mod models;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use models::TLUser;

pub struct DBManager {
    connection: PgConnection,
}


type Result<T> = std::result::Result<T, ()>;
type DieselResult<U> = std::result::Result<U, diesel::result::Error>;

impl DBManager {
    pub fn new(db_url: &str) -> Result<DBManager> {
        Ok(DBManager {connection: PgConnection::establish(db_url).unwrap()})
    }

    pub fn create_user(&self, id: i64, name_: String, username_: Option<String>) -> Result<()>{
        use scheme::users;
        use scheme::users::dsl::*;
        
        let q: DieselResult<i64> = users
        .filter(chat_id.eq(id))
        .select(chat_id)
        .first(&self.connection);

        if q.is_ok() {
            return Ok(())
        }
    
        let user = TLUser { 
            chat_id: id,
            name: name_,
            username: username_
        };

        let row: DieselResult<TLUser> = diesel::insert_into(users::table)
        .values(&user)
        .get_result(&self.connection);
        
        if row.is_err() {
            return Err(())
        }

        log::info!("User Created Successfully => {:?}", row.unwrap());

        Ok(())
    }
}


pub mod serde_schemes {
    use serde::Deserialize;


    #[derive(Deserialize, Debug)]
    pub struct Variant {
        pub bitrate: Option<i32>,
        pub content_type: String,
        pub url: String
    }
    
    #[derive(Deserialize, Debug)]
    pub struct VideoInfo {
        pub variants: Vec<Variant>
    }

    #[derive(Deserialize, Debug)]
    pub struct Media {
        pub video_info: Option<VideoInfo>,
        pub r#type: String,
        pub media_url_https: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    pub struct ExtendenEntities {
        pub media: Vec<Media>
    }

    #[derive(Deserialize, Debug)]
    pub struct User {
        pub id_str: String,
        pub name: String,
        pub screen_name: String
    }

    #[derive(Deserialize, Debug)]
    pub struct Body {
        pub extended_entities: Option<ExtendenEntities>,
        pub full_text: Option<String>,
        pub user: User
    }

    #[derive(Deserialize, Debug)]
    pub struct GuestToken {
        pub guest_token: String
    }

    #[derive(Deserialize, Debug)]
    pub struct SpaceObject {
        pub data: SpaceData,
    }

    #[derive(Deserialize, Debug)]
    pub struct SpaceData {
        pub audioSpace: AudioSpace,
    }

    #[derive(Deserialize, Debug)]
    pub struct AudioSpace {
        pub metadata: SpaceMetadata,
    }

    #[derive(Deserialize, Debug)]
    pub struct SpaceMetadata {
        pub media_key: String,
        pub token: Option<String>
    }

    #[derive(Deserialize, Debug)]
    pub struct SpacePlaylist {
        pub source: PlaylistSource
    }

    #[derive(Deserialize, Debug)]
    pub struct PlaylistSource {
        pub location: String
    }
}