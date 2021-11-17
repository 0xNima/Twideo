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