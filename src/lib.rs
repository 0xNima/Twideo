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
        if db_url.is_empty() {
            return Err(());
        }
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

    #[derive(Deserialize, Debug, Clone)]
    pub struct Variant {
        pub bit_rate: Option<i32>,
        pub content_type: String,
        pub url: String
    }

    #[derive(Deserialize, Debug)]
    pub struct User {
        pub id_str: String,
        pub name: String,
        pub screen_name: String
    }

    #[derive(Deserialize, Debug)]
    pub struct Media {
        pub r#type: String,
        pub preview_image_url: Option<String>,
        pub variants: Option<Vec<Variant>>,
        pub url: Option<String>
    }

    #[derive(Deserialize, Debug)]
    pub struct TwitterUser {
        pub name: String,
        pub username: String
    }

    #[derive(Deserialize, Debug)]
    pub struct MultimediaIncludes {
        pub media: Option<Vec<Media>>,
        pub users: Vec<TwitterUser>
    }
    
    #[derive(Deserialize, Debug)]
    pub struct MultimediaData {
        pub text: Option<String>,
        pub conversation_id: Option<String>,
        pub author_id: Option<String>
    }

    #[derive(Deserialize, Debug)]
    pub struct MultimediaBody {
        pub includes: Option<MultimediaIncludes>,
        pub data: MultimediaData
    }


    #[derive(Deserialize, Debug)]
    pub struct ThreadSearchData {
        pub id: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct ThreadSearchResult {
        pub data: Vec<ThreadSearchData>
    }
}