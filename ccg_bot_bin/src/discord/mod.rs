//!This way be Discord

//crate
use crate::config::Config;
#[cfg(test)]
use crate::env;
//skip reordering to allow easy reference to verbosity(from least to most)
#[rustfmt::skip]
use crate::{/*warn, */info, debug, trace};
#[cfg(any(feature = "discord", feature = "full"))]
use crate::utils::commandinteraction::CommandInteraction;
// #[cfg(any(feature = "discord", feature = "full"))]
// use crate::utils::TestUser;

#[cfg(all(any(feature = "discord", feature = "full"), test))]
use serde::ser::SerializeSeq;

//serenity
#[cfg(all(any(feature = "discord", feature = "full"), test))]
use serenity::all::ShardId;
use serenity::all::{
    Client, Context, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler,
    GatewayIntents, GuildId, Interaction, Message, Ready,
};
use serenity::async_trait;
//use serenity::model::prelude::*;

//std
use std::error;
use std::fmt;

//re-exports
#[cfg(all(any(feature = "discord", feature = "full"), not(test)))]
mod builders;
#[cfg(all(any(feature = "discord", feature = "full"), test))]
pub mod builders;
#[cfg(any(feature = "discord", feature = "full"))]
use self::builders::discordembed::DiscordEmbed;

#[doc(hidden)]
mod cache;
#[cfg(all(any(feature = "discord", feature = "full"), not(test)))]
mod commands;
#[cfg(all(any(feature = "discord", feature = "full"), test))]
pub mod commands;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref INTENTS: GatewayIntents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_PRESENCES;
}

#[derive(Debug)]
pub struct Handler(pub Config);

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction.clone() {
            debug!("[mod#L61] {:?}", &command.data);
            let command_interaction = CommandInteraction::from(interaction);
            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command_interaction, &ctx).await),
                "id" => Some(commands::id::run(&command_interaction, &ctx).await),
                _ => Some(DiscordEmbed::not_implimented()),
            };

            if let Some(ref _why) = content {
                let data = CreateInteractionResponseMessage::new().add_embed(content.expect(""));
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready<'a>(&'a self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        let gid = GuildId::new(
            self.0.discord_guildid.clone().parse().expect("guildid must be an integer"),
        );

        let commands = gid
            .set_commands(&ctx.http, vec![commands::ping::register(), commands::id::register()])
            .await;
        let mut vec_commands = Vec::new();
        let _ = commands.unwrap().drain(..).for_each(|c| vec_commands.push(c.name));
        info!("I now have the following guild slash commands: {:?}", vec_commands);
    }

    ///This prints every message the bot can see, in the format:
    ///<pre>[Channel] Author: Message</pre>
    async fn message<'a>(&'a self, ctx: Context, msg: Message) {
        // let channel_name: String = match ctx.cache.guild_channel(msg.channel_id) {
        let channel_name: String = match ctx.cache.channel(msg.channel_id) {
            Some(channel) => channel.name.clone(),
            None => return,
        };
        println!("[Discord / #{}] {}: {}", channel_name, msg.author.name, msg.content);
    }
}

#[derive(Debug)]
pub enum DiscordErr {
    JoinError(tokio::task::JoinError),
    Serenity(serenity::Error),
    VarErr(std::env::VarError),
}

impl fmt::Display for DiscordErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            DiscordErr::JoinError(ref err) => write!(f, "Join error: {err}"),
            DiscordErr::Serenity(ref err) => write!(f, "Serenity error: {err}"),
            DiscordErr::VarErr(ref err) => write!(f, "Var error: {err}"),
        }
    }
}

impl error::Error for DiscordErr {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            // N.B. Both of these implicitly cast `err` from their concrete
            // types (either `&serenity::Error` or `&env::VarError`)
            // to a trait object `&Error`. This works because both error types
            // implement `Error`.
            DiscordErr::JoinError(ref err) => Some(err),
            DiscordErr::Serenity(ref err) => Some(err),
            DiscordErr::VarErr(ref err) => Some(err),
        }
    }
}

impl From<tokio::task::JoinError> for DiscordErr {
    fn from(err: tokio::task::JoinError) -> DiscordErr {
        DiscordErr::JoinError(err)
    }
}

impl From<serenity::Error> for DiscordErr {
    fn from(err: serenity::Error) -> DiscordErr {
        DiscordErr::Serenity(err)
    }
}

impl From<std::env::VarError> for DiscordErr {
    fn from(err: std::env::VarError) -> DiscordErr {
        DiscordErr::VarErr(err)
    }
}

pub async fn new(config: Config) -> Result<Handler, serenity::Error> {
    let discord_token = config.discord_token.clone();

    //shadow this because test
    #[cfg(test)]
    let discord_token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| {
        "AbcDEFGhJkl0MnO1PQRsTUvx.Abcdef.AbCDefgHiJkLMNOpqrSTU0vWXy1".to_string()
    });

    //let intents: GatewayIntents = GatewayIntents::non_privileged()
    //    | GatewayIntents::MESSAGE_CONTENT
    //    | GatewayIntents::GUILD_MEMBERS
    //    | GatewayIntents::GUILD_PRESENCES;

    // mark these allows to not get a warning in tests::discord::it_works
    #[allow(unused_variables)]
    #[allow(unused_mut)]
    let mut client: Client = Client::builder(discord_token, *INTENTS)
        .event_handler(Handler(config.clone()))
        .await
        .expect("Error creating client");

    match client.start().await {
        Ok(a) => {
            #[cfg(test)]
            {
                dbg!(&a);
                dbg!(&config);
            }
            let _ = a;

            Ok(Handler(config))
        },
        Err(e) => Err(e),
    }
}

#[cfg(test)]
fn default_config() -> std::result::Result<Handler, serenity::Error> {
    std::result::Result::Ok(Handler(Config::default()))
}

#[cfg(all(any(feature = "discord", feature = "full"), test))]
#[derive(Debug)]
struct TestShardInfo {
    pub id: ShardId,
    pub total: u32,
}

//Seems sad to have to re-impl these but I couldn't get `#[serde(remote = "...")]` to work
#[cfg(all(any(feature = "discord", feature = "full"), test))]
impl<'de> serde::Deserialize<'de> for TestShardInfo {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        <(u32, u32)>::deserialize(deserializer)
            .map(|(id, total)| TestShardInfo { id: ShardId(id), total })
    }
}

//Seems sad to have to re-impl these but I couldn't get `#[serde(remote = "...")]` to work
#[cfg(all(any(feature = "discord", feature = "full"), test))]
impl serde::Serialize for TestShardInfo {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.id.0)?;
        seq.serialize_element(&self.total)?;
        seq.end()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::utils::TestUser;
    use crate::{
        config::Config, tests::discord::CurrentUser, utils::json::prelude::from_str, StdResult,
    };
    use error::Error;
    use serde::{Deserialize, Serialize};
    use serenity::all::{
        ApplicationFlags, ApplicationId, Cache, ChannelType as SerenityChannelType,
        CommandInteraction as SerenityCommandInteraction, ConnectionStage, Http,
        PartialCurrentApplicationInfo, PresenceData, Ready, Shard, ShardId, ShardInfo,
        ShardManager, ShardManagerOptions, ShardMessenger, ShardRunner, ShardRunnerOptions,
        UnavailableGuild,
    };
    use serenity::futures::{SinkExt, StreamExt};
    use serenity::gateway::WsClient;
    use serenity::prelude::{Mutex, RwLock, TypeMap};
    use std::{
        env,
        hash::Hash,
        sync::{Arc, OnceLock},
        time::Instant,
    };
    use tokio::{
        net::TcpListener,
        task::{yield_now, AbortHandle},
    };
    use tokio_websockets::ServerBuilder;

    static WS_ABORT_HANDLE: OnceLock<AbortHandle> = OnceLock::new();
    static WS_SERVER: OnceLock<TcpListener> = OnceLock::new();

    async fn test_ws(url: String) -> &'static AbortHandle {
        let server_result = TcpListener::bind(url.as_str()).await.map_err(|_| {});
        dbg!(&server_result);
        let server =
            WS_SERVER.get_or_init(|| server_result.expect("Couldn't bind to a socket at {url}\n"));
        let server_handle = tokio::spawn(async move {
            while let Ok((stream, _)) = server.accept().await {
                let mut ws_stream = ServerBuilder::new().accept(stream).await.expect("");
                tokio::spawn(async move {
                    while let Some(Ok(msg)) = ws_stream.next().await {
                        if msg.is_text() || msg.is_binary() {
                            ws_stream.send(msg).await.expect("");
                        }
                    }
                });
            }
            yield_now().await;
        });
        WS_ABORT_HANDLE.get_or_init(|| server_handle.abort_handle())
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct LocalAppInfo {
        pub id: ApplicationId,
        pub flags: ApplicationFlags,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub(crate) struct LocalReady {
        #[serde(rename = "v")]
        pub version: u8,
        pub user: CurrentUser,
        pub guilds: Vec<UnavailableGuild>,
        pub session_id: String,
        pub resume_gateway_url: String,
        pub shard: Option<ShardInfo>,
        pub application: PartialCurrentApplicationInfo,
    }

    pub struct LocalShard {
        pub client: WsClient,
        presence: PresenceData,
        last_heartbeat_sent: Option<Instant>,
        last_heartbeat_ack: Option<Instant>,
        heartbeat_interval: Option<std::time::Duration>,
        application_id_callback: Option<Box<dyn FnOnce(ApplicationId) + Send + Sync>>,
        last_heartbeat_acknowledged: bool,
        seq: u64,
        session_id: Option<String>,
        shard_info: ShardInfo,
        stage: ConnectionStage,
        pub started: Instant,
        pub token: String,
        ws_url: Arc<Mutex<String>>,
        pub intents: GatewayIntents,
    }

    impl fmt::Debug for LocalShard {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("LocalShard")
                .field("client", &"<serenity::gateway::ws::WsClient>")
                .field("presence", &self.presence)
                .field("last_heartbeat_sent", &self.last_heartbeat_sent)
                .field("last_heartbeat_ack", &self.last_heartbeat_ack)
                .field("heartbeat_interval", &self.heartbeat_interval)
                .field("application_id_callback", &"{closure}")
                .field("last_heartbeat_acknowledged", &self.last_heartbeat_acknowledged)
                .field("seq", &self.seq)
                .field("session_id", &self.session_id)
                .field("shard_info", &self.shard_info)
                .field("stage", &self.stage)
                .field("started", &self.started)
                .field("token", &"[REDACTED]")
                .field("ws_url", &self.ws_url)
                .field("intents", &self.intents)
                .finish()
        }
    }

    impl From<Shard> for LocalShard {
        fn from(value: Shard) -> Self {
            // all this because the field client moves *sigh*
            let presence = value.presence().clone();
            let last_heartbeat_sent = value.last_heartbeat_sent();
            let last_heartbeat_ack = value.last_heartbeat_ack();
            let heartbeat_interval = value.heartbeat_interval();
            let last_heartbeat_acknowledged = value.last_heartbeat_acknowledged();
            let seq = value.seq();
            let session_id = value.session_id().cloned();
            let shard_info = value.shard_info();
            let stage = value.stage();
            let started = value.started;
            let token = String::from(value.token.as_str());
            let intents = value.intents;

            Self {
                client: value.client,
                presence,
                last_heartbeat_sent,
                last_heartbeat_ack,
                heartbeat_interval,
                // create our own closure here because we can't call the underlying one in test
                // anyway
                application_id_callback: Some(Box::new(|_a| {})),
                last_heartbeat_acknowledged,
                seq,
                session_id,
                shard_info,
                stage,
                started,
                token,
                // ws_url doesn't have a getter directly
                ws_url: Arc::new(Mutex::new(String::from("ws://127.0.0.1:8000/"))),
                intents,
            }
        }
    }

    pub async fn setup_vars() -> (&'static AbortHandle, Handler, Context, Ready) {
        let cache = Cache::new();
        let c = Arc::new(cache);
        let token = Config::new().discord_token;
        let http = Arc::new(Http::new(&token));
        let raw_ws_url = String::from("127.0.0.1:8000");
        let test_server = test_ws(raw_ws_url.clone()).await;
        let ws_url_with_proto = String::from("ws://") + raw_ws_url.as_str() + "/";
        let ws_url = Arc::new(Mutex::new(ws_url_with_proto));
        let manager_options = ShardManagerOptions {
            data: Arc::new(RwLock::new(TypeMap::new())),
            event_handlers: vec![],
            raw_event_handlers: vec![],
            shard_index: 0u32,
            shard_init: 0u32,
            shard_total: 1u32,
            ws_url: ws_url.clone(),
            cache: c.clone(),
            http: http.clone(),
            intents: GatewayIntents::default(),
            presence: None,
        };
        let manager = ShardManager::new(manager_options).0;
        let _ = manager.initialize();
        let test_shard_info = TestShardInfo { id: ShardId(0u32), total: 1u32 };
        let test_shard_info_string = serde_json::to_string(&test_shard_info).expect("");
        let shard_info: ShardInfo = serde_json::from_str(&test_shard_info_string).expect("");
        let shard =
            Shard::new(ws_url, "", shard_info, GatewayIntents::default(), None).await.expect("");
        let runner_options = ShardRunnerOptions {
            data: Default::default(),
            event_handlers: vec![],
            raw_event_handlers: vec![],
            manager,
            shard,
            cache: c,
            http,
        };
        let runner = ShardRunner::new(runner_options);
        let handler_context = Context {
            data: Arc::new(RwLock::new(TypeMap::new())),
            shard: ShardMessenger::new(&runner),
            shard_id: ShardId(0_u32),
            http: Arc::new(Http::new("")),
            cache: Arc::new(Cache::new()),
        };
        let handler = Handler(Config::default());
        let user_obj = TestUser::default();
        let current_user_str = serde_json::to_string(&user_obj)
            .expect("Upstream may have broken our struct due to inheritance");
        let current_user: CurrentUser = serde_json::from_str(&current_user_str)
            .expect("Upstream may have broken our struct due to inheritance");
        let app_id = ApplicationId::new(1_u64);
        let app_flag = ApplicationFlags::default();
        let partial_app_info: PartialCurrentApplicationInfo = serde_json::from_str(
            &serde_json::to_string(&LocalAppInfo { id: app_id, flags: app_flag })
                .expect("Upstream may have broken our struct due to inheritance"),
        )
        .expect("Upstream may have broken our struct due to inheritance");
        let ready: Ready = serde_json::from_str(
            &serde_json::to_string(&LocalReady {
                version: 10u8,
                user: current_user,
                guilds: vec![],
                session_id: Default::default(),
                resume_gateway_url: Default::default(),
                shard: Some(shard_info),
                application: partial_app_info,
            })
            .expect("Upstream may have broken our struct due to inheritance"),
        )
        .expect("Upstream may have broken our struct due to inheritance");

        (test_server, handler, handler_context, ready)
    }

    #[tokio::test]
    async fn handler_interaction_create() -> Result<(), crate::DiscordError> {
        // use crate::tests::discord::test_logging;
        // use crate::tests::discord::test_logging::Level;
        // test_logging::init(Level::Debug);
        let (test_server, handler, handler_context, ready) = setup_vars().await;
        let handler_interaction_command_ping_str = r#"
            {
                "id": "1",
                "application_id": "2",
                "type": 2,
                "data": {
                    "id": "1",
                    "name": "ping",
                    "type": 255,
                    "resolved": {},
                    "options": [],
                    "target_id": null
                },
                "channel": {
                    "id": 3,
                    "name": "Test Private Message",
                    "type": 1,
                    "permissions": null
                },
                "channel_id": "3",
                "user": {
                    "id": "4",
                    "avatar": null,
                    "bot": false,
                    "discriminator": "0000",
                    "username": "",
                    "public_flags": null,
                    "banner": null,
                    "accent_color": null
                },
                "token": "DUMMYTOKEN",
                "version": 0,
                "app_permissions": "104320065",
                "locale": "en-US",
                "guild_locale": "en-US",
                "entitlements": []
            }
        "#
        .replace(
            "DUMMYTOKEN",
            env::var("DISCORD_TOKEN")
                .expect("Please set DISCORD_TOKEN in your environment")
                .as_str(),
        );
        let handler_interaction_command_id_str = r#"
            {
                "id": "1",
                "application_id": "2",
                "type": 2,
                "data": {
                    "id": "1",
                    "name": "id",
                    "type": 255,
                    "resolved": {
                        "users": {
                            "379001295744532481": {
                                "id": "379001295744532481",
                                "avatar": "d41d8cd98f00b204e9800998ecf8427e",
                                "bot": true,
                                "discriminator": 0,
                                "username": "Test",
                                "public_flags": null,
                                "banner": null,
                                "accent_colour": null
                            }
                        }
                    },
                    "options": [
                        {
                            "name": "id",
                            "value": "379001295744532481",
                            "type": 6,
                            "options": [],
                            "resolved": {
                                "User": [
                                    {
                                        "id":"379001295744532481",
                                        "avatar": "d41d8cd98f00b204e9800998ecf8427e",
                                        "bot": true,
                                        "discriminator": "0000",
                                        "username": "Test",
                                        "public_flags": null,
                                        "banner": null,
                                        "accent_color": null
                                    },
                                    {
                                        "deaf": false,
                                        "joined_at": "2015-10-03T13:52:36.422Z",
                                        "mute": false,
                                        "nick": null,
                                        "roles": [],
                                        "pending": false,
                                        "premium_since": null,
                                        "guild_id": null,
                                        "user": null,
                                        "permissions": "0"
                                    }
                                ]
                            },
                            "focused":false
                        }                        
                    ],
                    "target_id": null
                },
                "channel": {
                    "id": 3,
                    "name": "Test Private Message",
                    "type": 1,
                    "permissions": null

                },
                "channel_id": "3",
                "user": {
                    "id": "4",
                    "avatar": null,
                    "bot": false,
                    "discriminator": "0000",
                    "username": "",
                    "public_flags": null,
                    "banner": null,
                    "accent_color": null
                },
                "token": "DUMMYTOKEN",
                "version": 0,
                "app_permissions": "104320065",
                "locale": "en-US",
                "guild_locale": "en-US",
                "entitlements": []
            }
        "#
        .replace(
            "DUMMYTOKEN",
            env::var("DISCORD_TOKEN")
                .expect("Please set DISCORD_TOKEN in your environment")
                .as_str(),
        );

        let _ = handler.ready(handler_context.clone(), /* Ready{ .. }*/ ready);
        //ping
        let handler_interaction_command_ping: SerenityCommandInteraction =
            from_str(&handler_interaction_command_ping_str).unwrap();
        let handler_interaction_ping = Interaction::Command(handler_interaction_command_ping);
        let _ = handler.interaction_create(handler_context.clone(), handler_interaction_ping).await;
        //id
        let handler_interaction_command_id: SerenityCommandInteraction =
            from_str(&handler_interaction_command_id_str).unwrap();
        dbg!(&handler_interaction_command_id);
        let handler_interaction_id = Interaction::Command(handler_interaction_command_id);
        dbg!(&handler_interaction_id);
        test_server.abort();
        eprintln!("test_server.abort()");
        let _ = handler.interaction_create(handler_context, handler_interaction_id).await;
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn unimplemented_handler_interaction_create() {
        // use crate::tests::discord::test_logging;
        // use crate::tests::discord::test_logging::Level;
        // test_logging::init(Level::Debug);
        let (test_server, handler, handler_context, ready) = setup_vars().await;
        let handler_interaction_command_never_str = r#"
            {
                "id": "1",
                "application_id": "2",
                "type": 2,
                "data": {
                    "id": "1",
                    "name": "💀",
                    "type": 255,
                    "resolved": {
                        "users": {
                            "379001295744532481": {
                                "id": "379001295744532481",
                                "avatar": "d41d8cd98f00b204e9800998ecf8427e",
                                "bot": true,
                                "discriminator": 0,
                                "username": "Test",
                                "public_flags": null,
                                "banner": null,
                                "accent_colour": null
                            }
                        }
                    },
                    "options": [],
                    "target_id": null
                },
                "channel_id": "3",
                "user": {
                    "id": "4",
                    "avatar": null,
                    "bot": false,
                    "discriminator": "0000",
                    "username": "",
                    "public_flags": null,
                    "banner": null,
                    "accent_color": null
                },
                "token": "DUMMYTOKEN",
                "version": 0,
                "app_permissions": "104320065",
                "locale": "en-US",
                "guild_locale": "en-US",
                "entitlements": []
            }
        "#;
        let _ = handler.ready(handler_context.clone(), ready);
        //unimplemented
        let handler_interaction_command_never: SerenityCommandInteraction =
            from_str(handler_interaction_command_never_str).expect("");
        let handler_interaction_never = Interaction::Command(handler_interaction_command_never);
        test_server.abort();
        eprintln!("test_server.abort()");
        let _ = handler.interaction_create(handler_context, handler_interaction_never).await;
    }

    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
    #[repr(u8)]
    pub enum ChannelType {
        Text = 0,
        Private = 1,
        Voice = 2,
        Category = 4,
        News = 5,
        NewsThread = 10,
        PublicThread = 11,
        PrivateThread = 12,
        Stage = 13,
        Directory = 14,
        Forum = 15,
        Unknown(u8),
    }

    impl Default for ChannelType {
        fn default() -> Self {
            ChannelType::Text
        }
    }

    impl From<SerenityChannelType> for ChannelType {
        fn from(value: SerenityChannelType) -> Self {
            let chantype = match value {
                SerenityChannelType::Text => ChannelType::Text,
                SerenityChannelType::Private => ChannelType::Private,
                SerenityChannelType::Voice => ChannelType::Voice,
                SerenityChannelType::Category => ChannelType::Category,
                SerenityChannelType::News => ChannelType::News,
                SerenityChannelType::NewsThread => ChannelType::NewsThread,
                SerenityChannelType::PublicThread => ChannelType::PublicThread,
                SerenityChannelType::PrivateThread => ChannelType::PrivateThread,
                SerenityChannelType::Stage => ChannelType::Stage,
                SerenityChannelType::Directory => ChannelType::Directory,
                SerenityChannelType::Forum => ChannelType::Forum,
                SerenityChannelType::Unknown(u) => ChannelType::Unknown(u),
                _ => unimplemented!("Unknown type {value:?}"),
            };
            chantype
        }
    }

    #[tokio::test]
    async fn handler_message() {
        let (test_server, handler, handler_context, _ready) = setup_vars().await;
        let message_str = r#"{
            "id": "1093709276008161320",
            "attachments": [],
            "author": {
                "id": "379001295744532481",
                "avatar": "d41d8cd98f00b204e9800998ecf8427e",
                "bot": true,
                "discriminator": "0000",
                "username": "Test",
                "public_flags": null,
                "banner": null,
                "accent_color": null
            },
            "channel_id": "3",
            "content": "Test content",
            "edited_timestamp": null,
            "embeds": [],
            "guild_id": "5",
            "type": 0,
            "member": {
                "deaf": false,
                "joined_at": "2023-04-01T01:00:00.000Z",
                "mute": false,
                "nick": null,
                "roles": [],
                "pending": false,
                "premium_since": null,
                "guild_id": null,
                "user": null,
                "permissions": null
            },
            "mention_everyone": false,
            "mention_roles": [],
            "mention_channels": [],
            "mentions": [],
            "nonce": "1093709276008161320",
            "pinned": false,
            "reactions": [],
            "timestamp": "2023-04-07T01:30:11.536Z",
            "tts": false,
            "webhook_id": null,
            "activity": null,
            "application": null,
            "message_reference": null,
            "flags": 0,
            "sticker_items": [],
            "referenced_message": null,
            "interaction": null,
            "components": []
        }"#;
        let message: Message = from_str(message_str).unwrap();
        let _ = handler.message(handler_context, message).await;
        test_server.abort();
        eprintln!("test_server.abort()");
    }

    #[test]
    fn discorderr_derive_debug() {
        let _ = format!("{:?}", DiscordErr::Serenity(serenity::Error::Other(&"Test error")));
        let _ = format!("{:?}", DiscordErr::VarErr(std::env::VarError::NotPresent));
    }

    #[test]
    fn discorderr_display() {
        let _ = format!("{}", DiscordErr::Serenity(serenity::Error::Other(&"Test error")));
        let _ = format!("{}", DiscordErr::VarErr(std::env::VarError::NotPresent));
    }

    #[test]
    fn discorderr_from_impls() {
        let _: DiscordErr = serenity::Error::Other(&"Test error").into();
        let _: DiscordErr = std::env::VarError::NotPresent.into();
    }

    #[test]
    fn impl_error_for_discorderr() {
        let d_err: DiscordErr = serenity::Error::Other(&"Test error").into();
        match StdResult::<(), DiscordErr>::Err(d_err).err() {
            Some(e) => {
                println!("Error: {e}");
                println!("Caused by: {}", e.source().unwrap());
            },
            _ => println!("No error"),
        };
    }
}
