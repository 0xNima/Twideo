use crate::scheme::users;

#[derive(Identifiable, Queryable, Insertable, Debug, PartialEq)]
#[table_name="users"]
#[primary_key(chat_id)]
pub struct TLUser {
    pub chat_id: i64,
    pub name: String,
    pub username: Option<String>
}