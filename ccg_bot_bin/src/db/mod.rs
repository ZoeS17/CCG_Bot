pub mod models;
pub mod schema;

use diesel::prelude::*;
use eyre::Context;

use self::models::*;
use crate::CONFIG;

fn errot_to_eyre(e: ConnectionError) -> eyre::Report {
    eyre::Report::new(e) 
}

fn establish_connection() -> eyre::Result<MysqlConnection> {
    let database_url = CONFIG.clone().database_url;
    MysqlConnection::establish(&database_url)
       .map_err(|error| {
           error_to_eyre(error)
       })
}

/// Pull a [TwitchUser] from the database by its username
pub fn find_twitch_user(un: String) -> eyre::Result<TwitchUser> {
    use self::schema::twitchuser::dsl::*;

    let connection = &mut establish_connection()?;
    twitchuser
        .filter(username.eq(un))
        .select(TwitchUser::as_select())
        .load(connection)
        .expect("Error loading twitchuser")
        .first().cloned()
        // .map(|tu| tu.clone())
        .ok_or(eyre::eyre!("Error selecting twitchuser"))
}

/// Pull a [TwitchUser] from the database by its Twitch user id
pub fn find_twitch_user_by_id(uid: u32) -> eyre::Result<TwitchUser> {
    use self::schema::twitchuser::dsl::*;

    let connection = &mut establish_connection()?;
    twitchuser
        .filter(tid.eq(uid))
        .select(TwitchUser::as_select())
        .limit(1)
        .load(connection)
        .context("Error loading twitchuser")?
        .first().cloned()
        // .map(|tu| tu.clone())
        .ok_or(eyre::eyre!("Error finding twitch user by id"))
}

/// Pull [DiscordUser] from database by a Discord username
pub fn find_discord_user(un: String) -> eyre::Result<DiscordUser> {
    use self::schema::discorduser::dsl::*;

    let connection = &mut establish_connection()?;

    discorduser
        .select(DiscordUser::as_select())
        .filter(username.eq(un))
        .load(connection)
        .context("error selecting discord user by username")?
        .first().cloned()
        // .map(|du| du.clone())
        .ok_or(eyre::eyre!("Unable to find first instance of the user"))
}

/// Pull [DiscordUser] from the database by its Discord id
///
/// See [Discord Docs](https://support.discord.com/hc/en-us/articles/206346498-Where-can-I-find-my-User-Server-Message-ID#h_01HRSTXPS5H5D7JBY2QKKPVKNA) for how to obtain a Discord id from a user.
pub fn find_discord_user_by_id(id: u64) -> eyre::Result<DiscordUser> {
    use self::schema::discorduser::dsl::*;

    let connection = &mut establish_connection()?;

    discorduser
        .select(DiscordUser::as_select())
        .filter(did.eq(id))
        .load(connection)
        .context("error selecting discord user by id")?
        .first().cloned()
        // .map(|du| du.clone())
        .ok_or(eyre::eyre!("Unable to find first instance of the user"))
}
/// Pull all discord ids from users table where the twitch_id is passed in
// N.B.: Purposely pulling from the database twice to avoid data integrity issues
//       and further binding requirements
pub fn find_discord_user_by_twitch_id(tid: u32) -> eyre::Result<Vec<DiscordUser>> {
    use self::schema::discorduser::dsl::*;
    use self::schema::users::dsl::*;

    let mut result: Vec<DiscordUser> = vec![];

    let connection = &mut establish_connection()?;
    let twitch_users: Vec<u64> = users
        .select(discord_id)
        .filter(twitch_id.eq(tid))
        .load(connection)
        .context("Error selecting users by twitch_id")?;
    for u in twitch_users {
        result.push(
            discorduser
                .select(DiscordUser::as_select())
                .filter(did.eq(u))
                .load(connection)
                .context("error selecting discord user by id")?
                .first().cloned()
                // .map(|du| du.clone())
                .ok_or(eyre::eyre!("Unable to find first instance of the user"))?,
        )
    }
    Ok(result)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn select_twitch_user_by_username() {
        let expected = TwitchUser { tid: 12345678_u32, username: String::from("testuser") };
        let user = String::from("testuser");
        let needle = find_twitch_user(user);
        assert_eq!(needle.ok().unwrap(), expected);
    }

    #[test]
    fn select_twitch_user_by_id() {
        let expected = TwitchUser { tid: 12345678_u32, username: String::from("testuser") };
        let user = 12345678_u32;
        let needle = find_twitch_user_by_id(user);
        assert_eq!(needle.ok().unwrap(), expected);
    }

    #[test]
    fn select_discord_user_by_username() {
        let expected = DiscordUser { did: 123456789012345_u64, username: String::from("testuser") };
        let user = String::from("testuser");
        let needle = find_discord_user(user);
        assert_eq!(needle.unwrap(), expected);
    }

    #[test]
    fn select_discord_user_by_id() {
        let expected = DiscordUser { did: 123456789012345_u64, username: String::from("testuser") };
        let user = 123456789012345_u64;
        let needle = find_discord_user_by_id(user);
        assert_eq!(needle.unwrap(), expected);
    }

    #[test]
    fn select_discord_user_by_twitch_id() {
        let expected = DiscordUser { did: 123456789012345_u64, username: String::from("testuser") };
        let user = 12345678_u32;
        let needle = find_discord_user_by_twitch_id(user).ok().unwrap();
        assert_eq!(needle.first().unwrap(), &expected);
    }
}
