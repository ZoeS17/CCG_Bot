//crate
use crate::discord::builders::discordembed::DiscordEmbed;
use crate::utils::commandinteraction::{CommandInteraction, CommandInteractionResolved};
use crate::utils::json::prelude::{from_str, json, to_string, Value};

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
    user::User,
    user::UserPublicFlags,
    Permissions, Timestamp,
};
use serenity::utils::Color;

//std
use std::sync::Arc;

#[test]
fn it_works() {
    use super::super::config::Config;
    use super::super::discord::*;
    let dc: Result<Handler, serenity::Error> = aw!(new(Config {
        discord_token: "".to_string(),
        discord_guildid: "".to_string(),
        #[cfg(feature = "twitch")]
        twitch_channels: vec!["".to_string()],
    }));
    let disc_bool: bool = dc.is_ok();
    assert!(disc_bool);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

impl Default for TestUser {
    fn default() -> Self {
        TestUser {
            id: UserId(379001295744532481),
            avatar: Some("072bcea1eedb39786002311d5619a398".to_string()),
            bot: true,
            discriminator: 6349,
            name: "Test User".to_string(),
            public_flags: Some(UserPublicFlags::default()),
            banner: None,
            accent_colour: Some(Color::new(0x500060_u32)),
        }
    }
}

#[test]
fn id_command() {
    use super::super::discord::commands::id;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[repr(u8)]
    pub(crate) enum Cdov {
        String(String),
        Integer(i64),
        Boolean(bool),
        User(User, Option<PartialMember>),
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
        pub user: Option<User>,
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

    let pm = s_pm;
    let user = TestUser {
        id: UserId(99001900190019001),
        avatar: Some("".to_string()),
        bot: false,
        discriminator: 9001,
        name: "Test User".to_string(),
        accent_colour: Some(Color::new(0x500060_u32)),
        public_flags: Some(UserPublicFlags::default()),
        banner: Some("https://cdn.discordapp.com/avatars/379001295744532481/072bcea1eedb39786002311d5619a398.webp?size=1024".to_string())
    };

    let user_str = to_string(&user).unwrap();
    let user_cast = from_str::<User>(&user_str).unwrap();
    let resolved_obj = CommandInteractionResolved::User(user_cast, pm.ok());
    let test_ci = CommandInteraction {
        name: "id".to_string(),
        value: Some(Value::from("99001900190019001".to_string())),
        kind: CommandOptionType::User,
        options: vec![],
        resolved: Some(resolved_obj),
        focused: false,
    };
    dbg!(&test_ci);
    let options = test_ci;
    let c = Arc::new(cache);
    dbg!(&options);
    dbg!(&c);
    let _run = id::run(&options, c);
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
    type Output = User;

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
                let user = serde_json::from_value::<User>(json!({
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
        user_avatar: Some("https://cdn.discordapp.com/avatars/379001295744532481/072bcea1eedb39786002311d5619a398.webp?size=1024".to_string()),
        user_discriminator: 6349,
        user_id: UserId(99001900190019001),
        bot_user: true,
        user_name: "Test User".to_string(),
    };
    cache.update(&mut update_message);

    #[derive(Clone)]
    struct User {
        id: u64,
        name: String,
        avatar: String,
    }
    impl User {
        fn face(&self) -> String {
            self.clone().avatar
        }
    }

    let test_user_public_flags = Default::default();
    let user = User {
        id: 99001900190019001,
        name: "Test User".to_string(),
        avatar: "https://cdn.discordapp.com/avatars/379001295744532481/072bcea1eedb39786002311d5619a398.webp?size=1024".to_string(),
    };
    let user_update = UserUpdateEvent {
        current_user: CurrentUser {
            id: UserId(user.id),
            avatar: Some(user.clone().avatar),
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
        .thumbnail(user.face())
        .color(Color::new(0x500060_u32))
        .build();
    embed.author(|a| a.name("Courtesy Call Bot".to_string()).url("https://cdn.discordapp.com/avatars/379001295744532481/072bcea1eedb39786002311d5619a398.webp?size=1024".to_string()));
    dbg!(embed);
    //Maybe add a deserialize embed with a serde_json::json!(embed) and a handcrafted
    //string inside an assert?
}
