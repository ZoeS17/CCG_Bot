//crate
use crate::config::Config;
use crate::discord::builders::discordembed::DiscordEmbed;
use crate::utils::commandinteraction::{
    CommandInteraction, CommandInteractionResolved, LocalCommandData,
};
use crate::utils::json::prelude::{createembed_to_json_map, from_str, to_string, Value};
use crate::utils::TestUser;
//use crate::{FixedArray, FixedString};

//serde
use serde::{Deserialize, Serialize};

//serenity
use serenity::all::{
    ApplicationId, Attachment, Cache, ChannelId, Color, Context, CreateEmbedAuthor, GatewayIntents,
    GuildId, Http, InteractionId, PartialChannel, PartialMember, Permissions, Role, RoleId, Shard,
    ShardId, ShardInfo, ShardManager, ShardManagerOptions, ShardMessenger, ShardRunner,
    ShardRunnerOptions, Timestamp, User as SerenityUser, UserId, UserPublicFlags,
    UserUpdateEvent as SerenityUserUpdateEvent,
};
use serenity::prelude::{RwLock, TypeMap};

//tokio
use tokio::sync::Mutex;

//std
use std::sync::Arc;

macro_rules! cdn {
    ($e:expr) => {
        concat!("https://cdn.discordapp.com", $e)
    };
}

// pub(crate) mod test_logging {
//     use std::env;

//     // Keep the in order of least verbose to most
//     #[rustfmt::skip]
//     pub enum Level {
//         Error,
//         Warn,
//         Info,
//         Debug,
//         Trace,
//     }

//     impl From<Level> for String {
//         fn from(value: Level) -> Self {
//             match value {
//                 Level::Error => String::from("error"),
//                 Level::Warn => String::from("warn"),
//                 Level::Info => String::from("info"),
//                 Level::Debug => String::from("debug"),
//                 Level::Trace => String::from("trace"),
//             }
//         }
//     }

//     pub fn init(level: Level) {
//         let level = Into::<String>::into(level);
//         if env::var(level.clone()).is_err() {
//             env::set_var("RUST_LOG", &level);
//             tracing_subscriber::fmt::init();
//         }
//     }
// }

#[test]
fn it_works() {
    use super::super::config::Config;
    use super::super::discord::*;
    // use test_logging::Level;
    // test_logging::init(Level::Error);
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
        #[cfg(any(feature = "twitch", feature = "full"))]
        twitch_redirect_url: "".to_string()
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
#[derive(Debug, Serialize)]
pub(self) struct TestShardInfo {
    #[serde(with = "test_shard_info_serde")]
    pub id: ShardId,
    pub total: u32,
}

pub(crate) mod test_shard_info_serde {
    use super::ShardId;
    use std::fmt;

    use serde::de::{Error, Visitor};
    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<ShardId, D::Error> {
        deserializer.deserialize_any(TestShardInfoVisitor)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: Serializer>(id: &ShardId, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&id)
    }

    struct TestShardInfoVisitor;

    impl TestShardInfoVisitor {
        pub fn get_inner(outer: ShardId) -> u32 {
            outer.0
        }
    }

    impl<'de> Visitor<'de> for TestShardInfoVisitor {
        type Value = ShardId;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a valid u32")
        }

        fn visit_u32<E: Error>(self, value: u32) -> Result<Self::Value, E> {
            Ok(ShardId(value))
        }

        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            let fuckme = value;
            dbg!(fuckme);
            Ok(ShardId(0_u32))
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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

// fn server() -> () {
//     use super::super::discord::Self;
// }

#[tokio::test]
async fn id_command() {
    use super::super::discord;
    use super::super::discord::commands::id;
    use serenity::all::Context;
    // use test_logging::Level;
    // test_logging::init(Level::Error);
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

    let user = TestUser::default();
    let user_str = to_string(&user).unwrap();
    let user_cast = from_str::<SerenityUser>(&user_str).unwrap();
    let resolved_obj = CommandInteractionResolved::User(user_cast.id);
    let test_ci = CommandInteraction {
        id: InteractionId::new(!0u64),
        application_id: ApplicationId::new(!0u64),
        data: serde_json::from_str(&serde_json::to_string(&LocalCommandData::default()).expect(""))
            .expect(""),
        guild_id: None,
        channel: None,
        channel_id: ChannelId::new(!0_u64),
        member: None,
        user: serde_json::from_str(&serde_json::to_string(&TestUser::default()).expect(""))
            .expect(""),
        token: String::from(""),
        version: 1u8,
        app_permissions: None,
        locale: String::from(""),
        guild_locale: None,
    };
    // let options = test_ci;
    // let cache = Cache::new();
    // let c = Arc::new(cache);
    // let c_clone = &*Arc::try_unwrap(c.clone()).unwrap_err();
    // let token = Config::new().discord_token;
    // let http = Arc::new(Http::new(&token));
    // let manager_options = ShardManagerOptions {
    //     data: Arc::new(RwLock::new(TypeMap::new())),
    //     event_handlers: vec![],
    //     raw_event_handlers: vec![],
    //     shard_index: 0u32,
    //     shard_init: 0u32,
    //     shard_total: 1u32,
    //     ws_url: Default::default(),
    //     cache: c.clone(),
    //     http: http.clone(),
    //     intents: GatewayIntents::default(),
    //     presence: None,
    // };
    // let manager = ShardManager::new(manager_options);
    // dbg!(&manager);
    // let _ = manager.0.initialize();
    // dbg!(&manager);
    // let binding = manager.0.runners.lock().await;
    // dbg!(&binding);
    // let mut shard_keys = dbg!(binding.keys());
    // let shard_id = *shard_keys.next().expect("");
    // drop(binding);
    // let test_shard_info = TestShardInfo { id: shard_id, total: 1u32 };
    // let test_shard_info_string = serde_json::to_string(&test_shard_info).expect("");
    // let shard_info: ShardInfo = serde_json::from_str(&test_shard_info_string).expect("");
    // let shard = Shard::new(
    //     Arc::new(Mutex::new(String::from(""))),
    //     "",
    //     shard_info,
    //     GatewayIntents::default(),
    //     None,
    // )
    // .await
    // .expect("");
    // let runner_options = ShardRunnerOptions {
    //     data: Default::default(),
    //     event_handlers: vec![],
    //     raw_event_handlers: vec![],
    //     manager: manager.0,
    //     shard,
    //     cache: c.clone(),
    //     http: http.clone(),
    // };
    // let runner = ShardRunner::new(runner_options);
    // let context = Context {
    //     data: Default::default(),
    //     http,
    //     cache: c.clone(),
    //     shard: ShardMessenger::new(&runner),
    //     shard_id,
    // };
    let config = Config::new();
    let discord_handle = discord::new(config.clone());
    match discord::new(config.clone()).await {
        Ok(h) => eprintln!("{:?}", h),
        Err(e) => eprintln!("{:?}", e),
    };
    // let run = id::run(&options, &context);
    // let mut roles = format!(
    //     "{:?}",
    //     s_pm.ok()
    //         .unwrap()
    //         .roles
    //         .drain(..)
    //         .map(|r| format!("{}", r.to_role_cached(c_clone).unwrap()))
    //         .collect::<Vec<_>>()
    // );
    // roles.retain(|c| c != '[');
    // roles.retain(|c| c != ']');
    // roles.retain(|c| c != '"');
    let roles = ""; // XXX: Remove me
    let embed = DiscordEmbed::new()
        .field("id", format!("`{}`", user.id), true)
        .field("name", format!("`{}`", user.name), true)
        .field("mention", format!("<@{}>", user.id), true)
        .field("roles", roles.to_string(), false)
        .thumbnail(cdn!("/embed/avatars/0.png").to_string())
        .color(Color::new(0x500060_u32))
        .title(format!("{}'s info (w/ guild roles)", user.name))
        .build();
    let author =
        CreateEmbedAuthor::new("".to_string()).url(cdn!("/embed/avatars/0.png").to_string());
    let embed = embed.clone().author(author);
    // assert_eq!(
    //     Value::from(createembed_to_json_map(run.await)),
    //     Value::from(createembed_to_json_map(embed))
    // );
}

#[tokio::test]
async fn no_member_id_command() {
    use super::super::discord::commands::id;
    // use test_logging::Level;
    // test_logging::init(Level::Error);
    let cache = Arc::new(Cache::new());
    let user = TestUser::default();
    let user_str = to_string(&user).unwrap();
    let user_cast = from_str::<SerenityUser>(&user_str).unwrap();
    let resolved_obj = CommandInteractionResolved::User(user_cast.id);
    let test_ci = CommandInteraction {
        id: InteractionId::new(!0u64),
        application_id: ApplicationId::new(!0u64),
        data: serde_json::from_str(&serde_json::to_string(&LocalCommandData::default()).expect(""))
            .expect(""),
        guild_id: None,
        channel: None,
        channel_id: ChannelId::new(!0_u64),
        member: None,
        user: serde_json::from_str(&serde_json::to_string(&TestUser::default()).expect(""))
            .expect(""),
        token: String::from(""),
        version: 1u8,
        app_permissions: None,
        locale: String::from(""),
        guild_locale: None,
    };
    let options = test_ci;
    let token = Config::new().discord_token;
    let http = Arc::new(Http::new(&token));
    let manager_options = ShardManagerOptions {
        data: Arc::new(RwLock::new(TypeMap::new())),
        event_handlers: vec![],
        raw_event_handlers: vec![],
        shard_index: 0u32,
        shard_init: 0u32,
        shard_total: 1u32,
        ws_url: Default::default(),
        cache: cache.clone(),
        http: http.clone(),
        intents: GatewayIntents::default(),
        presence: None,
    };
    let manager = ShardManager::new(manager_options);
    let shard_id = *manager.0.runners.lock().await.keys().next().expect("");
    let test_shard_info = TestShardInfo { id: shard_id, total: 1u32 };
    let test_shard_info_string = serde_json::to_string(&test_shard_info).expect("");
    let shard_info: ShardInfo = serde_json::from_str(&test_shard_info_string).expect("");
    let shard = Shard::new(
        Arc::new(Mutex::new(String::from(""))),
        "",
        shard_info,
        GatewayIntents::default(),
        None,
    )
    .await
    .expect("");
    let runner_options = ShardRunnerOptions {
        data: Default::default(),
        event_handlers: vec![],
        raw_event_handlers: vec![],
        manager: manager.0,
        shard,
        cache: cache.clone(),
        http: http.clone(),
    };
    let runner = ShardRunner::new(runner_options);
    let context = Context {
        data: Default::default(),
        shard: ShardMessenger::new(&runner),
        shard_id: ShardId(0_u32),
        http,
        cache,
    };
    let run = id::run(&options, &context);
    let user = User { id: 0, name: "".to_string(), avatar: "0".to_string() };
    let embed = DiscordEmbed::new()
        .field("id", format!("`{}`", user.id), true)
        .field("name", format!("`{}`", user.name), true)
        .field("mention", format!("<@{}>", user.id), true)
        .thumbnail(cdn!("/embed/avatars/0.png").to_string())
        .color(Color::new(0x500060_u32))
        .title(format!("{}'s info", user.name))
        .build();
    let author =
        CreateEmbedAuthor::new("".to_string()).url(cdn!("/embed/avatars/0.png").to_string());
    let embed = embed.clone().author(author);
    assert_eq!(
        Value::from(createembed_to_json_map(run.await)),
        Value::from(createembed_to_json_map(embed))
    );
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct UserUpdateEvent {
    pub current_user: CurrentUser,
}

// https://serenity-rs.github.io/serenity/current/serenity/cache/trait.CacheUpdate.html#examples
/*
pub(crate) struct TestUserUpdate {
    user_avatar: Option<serenity::all::ImageHash>,
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
        match cache.users().get(&self.user_id) {
            Some(entry) => {
                let user: &mut serenity::model::prelude::User = &mut entry.to_owned();
                let old_user = user.clone();

                user.bot = self.bot_user;
                user.discriminator = NonZeroU16::new(self.user_discriminator);
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
            None => {
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

                let c = *cache;
                let t: () = c;

                // There was no old copy, so return None.
                None
            },
        }
    }
}
*/

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentUser {
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
// TODO: Un-nref this after I can figure out what the hades I'm doing wrong
/*
struct LocalShardMessenger(pub Rc<ShardMessenger>);

impl PartialEq for LocalShardMessenger {
    fn eq(&self, other: &Self) -> bool {
        // since ShardManager doesn't impl Eq nor allow us
        // access to the fields, compare these as raw pointers
        if &self.0 as *const _ == &other.0 as *const _ {
            return true;
        }
        false
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

struct LocalShardRunnerInfo(pub Rc<ShardRunnerInfo>);

impl PartialEq for LocalShardRunnerInfo {
    fn eq(&self, other: &Self) -> bool {
        let stage = self.0.stage == other.0.stage;
        let latency = self.0.latency == other.0.latency;
        let runner_tx = LocalShardMessenger(Rc::new(self.0.runner_tx.clone()))
            == LocalShardMessenger(Rc::new(other.0.runner_tx.clone()));
        if stage && latency && runner_tx {
            return true;
        }
        false
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Into<ShardRunnerInfo> for LocalShardRunnerInfo {
    fn into(self) -> ShardRunnerInfo {
        Rc::<ShardRunnerInfo>::into_inner(self.0)
            .expect("Unable to extract ShardRunnerInfo from LocalShardRunnerInfo")
    }
}

struct LocalShardId(pub ShardId);

impl PartialEq for LocalShardId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Into<ShardId> for LocalShardId {
    fn into(self) -> ShardId {
        self.0
    }
}

struct Private<'up> {
    id: ShardId,
    stage: serenity::gateway::ConnectionStage,
    runner_info: &'up ShardRunnerInfo,
    messenger: ShardMessenger,
}

struct LocalShardManager<'up>(pub &'up ShardManager, &'up mut Private<'up>);

impl<'up> LocalShardManager<'up> {
    fn _set_stage(&mut self, stage: serenity::gateway::ConnectionStage) {
        self.1.stage = stage;
    }

    fn _set_runner_info(&mut self, runner_info: &'up ShardRunnerInfo) {
        self.1.runner_info = runner_info;
    }

    fn _set_messenger(&mut self, messenger: ShardMessenger) {
        self.1.messenger = messenger;
    }

    fn _set_shard_id(&mut self, id: ShardId) {
        self.1.id = id;
    }

    fn _set_shard_id_u32(&mut self, id: u32) {
        self._set_shard_id(ShardId(id));
    }

    async fn shards_instantiated(&self) -> Vec<ShardId> {
        self.0.shards_instantiated().await
    }
    fn intents(&self) -> GatewayIntents {
        self.0.intents()
    }

    fn messenger(&mut self) -> &ShardMessenger {
        &self.1.messenger
    }

    fn runners(
        &mut self,
    ) -> Arc<serenity::prelude::Mutex<HashMap<ShardId, ShardRunnerInfo>>> {
        let shard_id: ShardId = self.1.id;
        let shard_runner_info =
            ShardRunnerInfo { latency: None, runner_tx: *self.messenger(), stage: self.1.stage };
        // let jail = OnceCell::new();
        // if let Ok(_) = jail.set(*&shard_runner_info) {
        //     unreachable!();
        // };
        let mut map = HashMap::new();
        self._set_runner_info(&shard_runner_info);
        map.insert(shard_id, shard_runner_info);
        Arc::new(serenity::prelude::Mutex::new(map))
    }
}

struct LocalRunner(Arc<serenity::prelude::Mutex<HashMap<ShardId, ShardRunnerInfo>>>);

impl PartialEq for LocalRunner {
    #[inline]
    fn eq(&self, other: &LocalRunner) -> bool {
        let rt = Runtime::new().unwrap();
        let self_inner: ShardRunnerInfo =
            *rt.block_on(async { self.0.lock().await }).get(&ShardId(0u32)).expect("");
        let other_inner: ShardRunnerInfo =
            *rt.block_on(async { other.0.lock().await }).get(&ShardId(0u32)).expect("");
        LocalShardRunnerInfo(Rc::new(self_inner)) == LocalShardRunnerInfo(Rc::new(other_inner))
    }
}
use tokio::runtime::Runtime;

impl PartialEq for LocalShardManager<'_> {
    fn eq(&self, other: &Self) -> bool {
        let rt = Runtime::new().unwrap();
        let self_shards = rt.block_on(async { self.shards_instantiated().await });
        let other_shards = rt.block_on(async { other.shards_instantiated().await });
        let shards_instantiated = self_shards == other_shards;
        let intents = self.intents() == other.intents();
        let self_runners = LocalRunner(self.runners());
        let other_runners = LocalRunner(other.runners());
        let runners = self_runners == other_runners;
        if shards_instantiated && intents && runners {
            return true;
        }
        false
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
*/

#[test]
fn embed_builder() {
    let test_user_public_flags = Default::default();
    let user = User {
        id: 379001295744532481,
        name: "Courtesy Call Bot".to_string(),
        avatar: cdn!("/avatars/379001295744532481/072bcea1eedb39786002311d5619a398.webp?size=1024")
            .to_string(),
    };
    let user_update = UserUpdateEvent {
        current_user: CurrentUser {
            id: UserId::new(user.id),
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
    let embed = DiscordEmbed::new()
        .field("id", format!("`{}`", user.id), true)
        .field("name", format!("`{}`", user.name), true)
        .field("mention", format!("<@{}>", user.id), true)
        .thumbnail(user.face())
        .color(Color::new(0x500060_u32))
        .title("Embed builder test")
        .build();
    let author = CreateEmbedAuthor::new("Courtesy Call Bot".to_string()).url(
        cdn!("/avatars/379001295744532481/072bcea1eedb39786002311d5619a398.webp?size=1024")
            .to_string(),
    );
    let _ = embed.author(author);
    //Maybe add a deserialize embed with a serde_json::json!(embed) and a handcrafted
    //string inside an assert?
}

// See above: On Nerfing
//#[tokio::test]
//async fn ping_command() {
//    use super::super::discord::commands::ping;

//    test_logging::init();

//    let cache = Cache::new();
//    let user = TestUser::default();
//    let user_str = to_string(&user).unwrap();
//    let user_cast = from_str::<SerenityUser>(&user_str).unwrap();
//    let resolved_obj = CommandInteractionResolved::User(user_cast.id);
//    let test_ci = CommandInteraction {
//        name: "ping".to_string(),
//        value: serenity::all::CommandDataOptionValue::User(TestUser::default().id),
//        kind: CommandOptionType::User,
//        options: vec![],
//        resolved: Some(resolved_obj),
//        focused: false,
//    };
//    let options = test_ci;
//    let c = Arc::new(cache);
//    let config = Config::new();
//    let token = config.discord_token;
//    let http = Arc::new(Http::new(&token));
//    let manager_options = ShardManagerOptions {
//        data: Arc::new(RwLock::new(TypeMap::new())),
//        event_handlers: vec![],
//        raw_event_handlers: vec![],
//        shard_index: 0u32,
//        shard_init: 1u32,
//        shard_total: 1u32,
//        ws_url: Default::default(),
//        cache: c.clone(),
//        http: http.clone(),
//        intents: GatewayIntents::default(),
//        presence: None,
//    };
//    let arc_manager = ShardManager::new(manager_options);
//    dbg!(&arc_manager);
//    let manager = arc_manager.0;
//    dbg!(&manager);
//    let man = manager.clone();
//    let intialized_manager = manager.initialize();
//    let id = ShardId(0u32);
//    let runner_info: ShardRunnerInfo = *(arc_manager.0.runners.lock().await.get(&id).expect(""));
//    let stage = ConnectionStage::Connected;
//    let messenger = runner_info.runner_tx;
//    // let jail = OnceCell::new();
//    // let Ok(_) = jail.set(runner_info);
//    let mut private = Private { id, stage, runner_info: &runner_info, messenger };
//    dbg!(&intialized_manager);
//    dbg!(&manager);
//    assert!(LocalShardManager(&manager, &mut private) == LocalShardManager(&man, &mut private));
//    let ret_value_from_shard_queuer = arc_manager.1;
//    dbg!(&ret_value_from_shard_queuer);
//    let runners = &manager.runners;
//    dbg!(&runners);
//    let shard_runner = runners.lock().await;
//    dbg!(&shard_runner);
//    let shard_id = shard_runner.keys().next().expect("");
//    let test_shard_info = TestShardInfo { id: *shard_id, total: 1u32 };
//    drop(shard_runner);
//    let test_shard_info_string = serde_json::to_string(&test_shard_info).expect("");
//    let shard_info: ShardInfo = serde_json::from_str(&test_shard_info_string).expect("");
//    let shard = Shard::new(
//        Arc::new(Mutex::new(String::from(""))),
//        "",
//        shard_info,
//        GatewayIntents::default(),
//        None,
//    )
//    .await
//    .expect("");
//    let runner_options = ShardRunnerOptions {
//        data: Default::default(),
//        event_handlers: vec![],
//        raw_event_handlers: vec![],
//        manager,
//        shard,
//        cache: c.clone(),
//        http: http.clone(),
//    };
//    let runner = ShardRunner::new(runner_options);
//    let context = Context {
//        data: Default::default(),
//        shard: ShardMessenger::new(&runner),
//        shard_id: ShardId(0_u32),
//        http,
//        cache: c,
//    };
//    let run = ping::run(&options, &context);
//    //test embed
//    let embed = DiscordEmbed::new()
//        .field("Greetings", "Program".to_string(), true)
//        .color(Color::new(0x500060_u32))
//        .thumbnail(
//            "https://cdn.discordapp.com/emojis/938514423155400804.webp?size=48&quality=lossless",
//        )
//        .title("Pong")
//        .build();
//    let author =
//        CreateEmbedAuthor::new("".to_string()).url(cdn!("/embed/avatars/0.png").to_string());
//    let embed = embed.clone().author(author);
//    //result
//    assert_eq!(
//        Value::from(createembed_to_json_map(run.await)),
//        Value::from(createembed_to_json_map(embed))
//    );
//}

#[test]
fn hanlder_debug() {
    use super::super::discord::*;
    use crate::config::Config;
    let _ = format!("{:?}", Handler(Config::default()));
}
