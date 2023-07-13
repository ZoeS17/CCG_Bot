//crate
use crate::discord::builders::discordembed::DiscordEmbed;
use crate::utils::commandinteraction::{CommandInteraction, CommandInteractionResolved};
use crate::utils::json::prelude::{from_str, hashmap_to_json_map, json, to_string, Value};

//dashmap
use dashmap::mapref::entry::Entry;

//serde
use serde::{Deserialize, Serialize};

//serenity
use serenity::cache::{Cache, CacheUpdate};
use serenity::model::{
    application::command::CommandOptionType,
    channel::{Attachment, PartialChannel},
    event::UserUpdateEvent as SerenityUserUpdateEvent,
    guild::{PartialMember, Role},
    id::{GuildId, RoleId, UserId},
    user::{User as SerenityUser, UserPublicFlags},
    Permissions, Timestamp,
};
use serenity::utils::Color;

//std
use std::sync::Arc;

macro_rules! cdn {
    ($e:expr) => {
        concat!("https://cdn.discordapp.com", $e)
    };
}

#[test]
fn it_works() {
    use super::super::config::Config;
    use super::super::discord::*;
    let dc: Result<Handler, serenity::Error> = aw!(new(Config {
        #[cfg(any(feature = "discord", feature = "full"))]
        discord_guildid: "".to_string(),
        #[cfg(any(feature = "discord", feature = "full"))]
        discord_token: "".to_string(),
        #[cfg(any(feature = "twitch", feature = "full"))]
        twitch_bot_name: "".to_string(),
        #[cfg(any(feature = "twitch", feature = "full"))]
        twitch_channels: vec!["".to_string()],
        #[cfg(any(feature = "twitch", feature = "full"))]
        twitch_client_id: "".to_string(),
        #[cfg(any(feature = "twitch", feature = "full"))]
        twitch_client_secret: "".to_string(),
    }));
    let disc_bool: bool = dc.is_ok();
    assert!(disc_bool);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    id: u64,
    name: String,
    avatar: String,
}
impl User {
    fn face(&self) -> String {
        self.clone().avatar
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TestUser {
    pub id: UserId,
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: bool,
    pub discriminator: u16,
    #[serde(rename = "username")]
    pub name: String,
    pub public_flags: Option<UserPublicFlags>,
    pub banner: Option<String>,
    #[serde(rename = "accent_color")]
    pub accent_colour: Option<Color>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub(crate) enum Cdov {
    String(String),
    Integer(i64),
    Boolean(bool),
    User(SerenityUser, Option<PartialMember>),
    Channel(PartialChannel),
    Role(Role),
    Number(f64),
    Attachment(Attachment),
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PM {
    #[serde(default)]
    pub deaf: bool,
    pub joined_at: Option<Timestamp>,
    #[serde(default)]
    pub mute: bool,
    pub nick: Option<String>,
    pub roles: Vec<RoleId>,
    #[serde(default)]
    pub pending: bool,
    pub premium_since: Option<Timestamp>,
    pub guild_id: Option<GuildId>,
    pub user: Option<SerenityUser>,
    pub permissions: Option<Permissions>,
}

impl From<PartialMember> for PM {
    fn from(pm: PartialMember) -> Self {
        Self {
            deaf: pm.deaf,
            joined_at: pm.joined_at,
            mute: pm.mute,
            nick: pm.nick,
            roles: pm.roles,
            pending: pm.pending,
            premium_since: pm.premium_since,
            guild_id: pm.guild_id,
            user: pm.user,
            permissions: pm.permissions,
        }
    }
}

#[test]
fn id_command() {
    use super::super::discord::commands::id;
    let cache = Cache::new();
    let s_pm = from_str::<PartialMember>(
        &to_string(&PM {
            deaf: false,
            joined_at: None,
            mute: false,
            nick: None,
            roles: vec![],
            pending: false,
            premium_since: None,
            guild_id: None,
            user: None,
            permissions: None,
        })
        .unwrap(),
    );

    let pm = s_pm.as_ref();
    let user = TestUser::default();

    let user_str = to_string(&user).unwrap();
    let user_cast = from_str::<SerenityUser>(&user_str).unwrap();
    let resolved_obj = CommandInteractionResolved::User(user_cast, pm.ok().cloned());
    let test_ci = CommandInteraction {
        name: "id".to_string(),
        value: Some(Value::from(TestUser::default().id.to_string())),
        kind: CommandOptionType::User,
        options: vec![],
        resolved: Some(resolved_obj),
        focused: false,
    };
    let options = test_ci;
    let c = Arc::new(cache);
    let c_clone = &*Arc::try_unwrap(c.clone()).unwrap_err();
    let run = id::run(&options, c);
    let mut roles = format!(
        "{:?}",
        s_pm.ok()
            .unwrap()
            .roles
            .drain(..)
            .map(|r| format!("{}", r.to_role_cached(c_clone).unwrap()))
            .collect::<Vec<_>>()
    );
    roles.retain(|c| c != '[');
    roles.retain(|c| c != ']');
    roles.retain(|c| c != '"');
    let mut embed = DiscordEmbed::new()
        .field("id", format!("`{}`", user.id), true)
        .field("name", format!("`{}`", user.name), true)
        .field("mention", format!("<@{}>", user.id), true)
        .field("roles", roles.to_string(), false)
        .thumbnail(cdn!("/embed/avatars/0.png").to_string())
        .color(Color::new(0x500060_u32))
        .title(format!("{}'s info (w/ guild roles)", user.name))
        .build();
    embed.author(|a| a.name("".to_string()).url(cdn!("/embed/avatars/0.png").to_string()));
    assert_eq!(Value::from(hashmap_to_json_map(run.0)), Value::from(hashmap_to_json_map(embed.0)));
}

#[test]
fn id_command_no_member() {
    use super::super::discord::commands::id;
    let cache = Cache::new();
    let user = TestUser::default();
    let user_str = to_string(&user).unwrap();
    let user_cast = from_str::<SerenityUser>(&user_str).unwrap();
    let resolved_obj = CommandInteractionResolved::User(user_cast, None);
    let test_ci = CommandInteraction {
        name: "id".to_string(),
        value: Some(Value::from(TestUser::default().id.to_string())),
        kind: CommandOptionType::User,
        options: vec![],
        resolved: Some(resolved_obj),
        focused: false,
    };
    let options = test_ci;
    let c = Arc::new(cache);
    let run = id::run(&options, c);
    let user = User { id: 0, name: "".to_string(), avatar: "0".to_string() };
    let mut embed = DiscordEmbed::new()
        .field("id", format!("`{}`", user.id), true)
        .field("name", format!("`{}`", user.name), true)
        .field("mention", format!("<@{}>", user.id), true)
        .thumbnail(cdn!("/embed/avatars/0.png").to_string())
        .color(Color::new(0x500060_u32))
        .title(format!("{}'s info", user.name))
        .build();
    embed.author(|a| a.name("".to_string()).url(cdn!("/embed/avatars/0.png").to_string()));
    assert_eq!(Value::from(hashmap_to_json_map(run.0)), Value::from(hashmap_to_json_map(embed.0)));
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct UserUpdateEvent {
    pub current_user: CurrentUser,
}

// https://serenity-rs.github.io/serenity/current/serenity/cache/trait.CacheUpdate.html#examples
pub(crate) struct TestUserUpdate {
    user_avatar: Option<String>,
    user_discriminator: u16,
    user_id: UserId,
    bot_user: bool,
    user_name: String,
}

// https://serenity-rs.github.io/serenity/current/serenity/cache/trait.CacheUpdate.html#examples
impl CacheUpdate for TestUserUpdate {
    // A copy of the old user's data, if it existed in the cache.
    type Output = SerenityUser;

    fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
        // If an entry for the user already exists, update its fields.
        match cache.users().entry(self.user_id) {
            Entry::Occupied(entry) => {
                let user: &mut serenity::model::prelude::User = &mut entry.get().to_owned();
                let old_user = user.clone();

                user.bot = self.bot_user;
                user.discriminator = self.user_discriminator;
                user.id = self.user_id;

                if user.avatar != self.user_avatar {
                    user.avatar = self.user_avatar.clone();
                }

                if user.name != self.user_name {
                    user.name = self.user_name.clone();
                }

                // Return the old copy for the user's sake.
                Some(old_user)
            },
            Entry::Vacant(entry) => {
                // We can convert a [`serde_json::Value`] to a User for test
                // purposes.
                let user = serde_json::from_value::<SerenityUser>(json!({
                    "id": self.user_id,
                    "avatar": self.user_avatar.clone(),
                    "bot": self.bot_user,
                    "discriminator": self.user_discriminator,
                    "username": self.user_name.clone(),
                }))
                .expect("Error making user");

                entry.insert(user);

                // There was no old copy, so return None.
                None
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CurrentUser {
    pub id: UserId,
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: bool,
    pub discriminator: u16,
    pub email: Option<String>,
    pub mfa_enabled: bool,
    #[serde(rename = "username")]
    pub name: String,
    pub verified: Option<bool>,
    pub public_flags: Option<UserPublicFlags>,
    pub banner: Option<String>,
    pub accent_colour: Option<Color>,
}

#[test]
fn embed_builder() {
    let cache = Cache::new();
    let _users = cache.users();
    let mut update_message = TestUserUpdate {
        user_avatar: Some(
            cdn!("/avatars/379001295744532481/072bcea1eedb39786002311d5619a398.webp?size=1024")
                .to_string(),
        ),
        user_discriminator: 6349,
        user_id: UserId(379001295744532481),
        bot_user: true,
        user_name: "Courtesy Call Bot".to_string(),
    };
    cache.update(&mut update_message);

    let test_user_public_flags = Default::default();
    let user = User {
        id: 379001295744532481,
        name: "Courtesy Call Bot".to_string(),
        avatar: cdn!("/avatars/379001295744532481/072bcea1eedb39786002311d5619a398.webp?size=1024")
            .to_string(),
    };
    let user_update = UserUpdateEvent {
        current_user: CurrentUser {
            id: UserId(user.id),
            avatar: Some(user.face()),
            bot: true,
            discriminator: 6349,
            email: Some(Default::default()),
            mfa_enabled: true,
            name: user.clone().name,
            verified: Some(false),
            public_flags: Some(test_user_public_flags),
            banner: None,
            accent_colour: Some(Color::new(0x500060_u32)),
        },
    };
    let user_update_str = serde_json::json!(&user_update).to_string();
    let _test_user_update = serde_json::from_str::<SerenityUserUpdateEvent>(&user_update_str);
    let mut embed = DiscordEmbed::new()
        .field("id", format!("`{}`", user.id), true)
        .field("name", format!("`{}`", user.name), true)
        .field("mention", format!("<@{}>", user.id), true)
        .thumbnail(user.face())
        .color(Color::new(0x500060_u32))
        .title("Embed builder test")
        .build();
    embed.author(|a| {
        a.name("Courtesy Call Bot".to_string()).url(
            cdn!("/avatars/379001295744532481/072bcea1eedb39786002311d5619a398.webp?size=1024")
                .to_string(),
        )
    });
    //Maybe add a deserialize embed with a serde_json::json!(embed) and a handcrafted
    //string inside an assert?
}

#[test]
fn ping_comand() {
    use super::super::discord::commands::ping;
    let cache = Cache::new();
    let user = TestUser::default();
    let user_str = to_string(&user).unwrap();
    let user_cast = from_str::<SerenityUser>(&user_str).unwrap();
    let resolved_obj = CommandInteractionResolved::User(user_cast, None);
    let test_ci = CommandInteraction {
        name: "ping".to_string(),
        value: Some(Value::from(TestUser::default().id.to_string())),
        kind: CommandOptionType::User,
        options: vec![],
        resolved: Some(resolved_obj),
        focused: false,
    };
    let options = test_ci;
    let c = Arc::new(cache);
    let run = ping::run(&options, c);
    //test embed
    let mut embed = DiscordEmbed::new()
        .field("Greetings", "Program".to_string(), true)
        .color(Color::new(0x500060_u32))
        .thumbnail(
            "https://cdn.discordapp.com/emojis/938514423155400804.webp?size=48&quality=lossless",
        )
        .title("Pong")
        .build();
    embed.author(|a| a.name("".to_string()).url(cdn!("/embed/avatars/0.png").to_string()));
    //result
    assert_eq!(Value::from(hashmap_to_json_map(run.0)), Value::from(hashmap_to_json_map(embed.0)));
}

#[test]
fn hanlder_debug() {
    use super::super::discord::*;
    use crate::config::Config;
    let _ = format!("{:?}", Handler(Config::default()));
}
