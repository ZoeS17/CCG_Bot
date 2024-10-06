use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Users {
    pub id: u32,
    pub uid: u32,
    pub discord_id: u64,
    pub twitch_id: u32,
}

#[derive(Clone, Debug, PartialEq, Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::discorduser)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct DiscordUser {
    pub did: u64,
    pub username: String,
}

#[derive(Clone, Debug, PartialEq, Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::twitchuser)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct TwitchUser {
    pub tid: u32,
    pub username: String,
}
