table! {
    users (chat_id) {
        chat_id -> Int8,
        name -> Varchar,
        username -> Nullable<Varchar>,
    }
}