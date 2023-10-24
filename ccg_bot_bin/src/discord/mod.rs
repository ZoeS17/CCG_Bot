//!This way be Discord

//crate
use crate::config::Config;
#[cfg(all(any(feature = "discord", feature = "full"), test))]
use crate::tests::discord::test_shard_info_serde;
use crate::utils::commandinteraction::CommandInteraction;

#[cfg(all(any(feature = "discord", feature = "full"), test))]
use serde::Serialize;

//serenity
use serenity::all::{
    CommandOptionType, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use serenity::async_trait;
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::prelude::*;
use serenity::prelude::{Client, Context, EventHandler};

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
            trace!("{:?}", &command.data);
            let opt: CommandInteraction = match command.data.options.get(0) {
                Some(o) => (*o).clone().into(),
                None => CommandInteraction {
                    name: "".to_string(),
                    value: CommandDataOptionValue::Unknown(!0u8),
                    kind: CommandOptionType::Unknown(!0u8),
                    options: vec![],
                    resolved: None,
                    focused: false,
                },
            };
            debug!("{:?}", &opt);
            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&opt, &ctx).await),
                "id" => Some(commands::id::run(&opt, &ctx).await),
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
        let channel_name: String = match ctx.cache.guild_channel(msg.channel_id) {
            Some(channel) => channel.name.clone(),
            None => return,
        };
        println!("[Discord / #{}] {}: {}", channel_name, msg.author.name, msg.content);
    }
}

#[derive(Debug)]
pub enum DiscordErr {
    Serenity(serenity::Error),
    VarErr(std::env::VarError),
}

impl fmt::Display for DiscordErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
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
            DiscordErr::Serenity(ref err) => Some(err),
            DiscordErr::VarErr(ref err) => Some(err),
        }
    }
}

impl From<std::env::VarError> for DiscordErr {
    fn from(err: std::env::VarError) -> DiscordErr {
        DiscordErr::VarErr(err)
    }
}

impl From<serenity::Error> for DiscordErr {
    fn from(err: serenity::Error) -> DiscordErr {
        DiscordErr::Serenity(err)
    }
}

pub async fn new(config: Config) -> Result<Handler, serenity::Error> {
    let dt = config.discord_token.clone();

    //let intents: GatewayIntents = GatewayIntents::non_privileged()
    //    | GatewayIntents::MESSAGE_CONTENT
    //    | GatewayIntents::GUILD_MEMBERS
    //    | GatewayIntents::GUILD_PRESENCES;

    // mark these allows to not get a warning in tests::discord::it_works
    #[allow(unused_variables)]
    #[allow(unused_mut)]
    let mut client: Client = Client::builder(dt, *INTENTS)
        .event_handler(Handler(config.clone()))
        .await
        .expect("Error creating client");

    // Wish there was a way to not have to conditonally compile here
    // but it seems `client.start().await` cause the test to go on
    // infinitly.
    #[cfg(not(test))]
    let c = match client.start().await {
        Ok(_) => Ok(Handler(config)),
        Err(e) => Err(e),
    };

    #[cfg(test)]
    let c = default_config();

    c
}

#[cfg(test)]
fn default_config() -> std::result::Result<Handler, serenity::Error> {
    std::result::Result::Ok(Handler(Config::default()))
}

#[cfg(all(any(feature = "discord", feature = "full"), test))]
#[derive(Debug, Serialize)]
pub(self) struct TestShardInfo {
    #[serde(with = "test_shard_info_serde")]
    pub id: ShardId,
    pub total: u32,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::config::Config;
    use crate::utils::json::prelude::from_str;
    use crate::StdResult;
    use error::Error;
    use serde::{Deserialize, Serialize};
    use serenity::all::{
        Cache, ChannelType as SerenityChannelType, CommandInteraction, Http, Shard, ShardId,
        ShardManager, ShardManagerOptions, ShardMessenger, ShardRunner, ShardRunnerOptions,
    };
    use serenity::prelude::{Mutex, RwLock, TypeMap};
    use std::hash::Hash;
    use std::sync::Arc;

    #[tokio::test]
    async fn handler_interaction_create() {
        let cache = Cache::new();
        let c = Arc::new(cache);
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
            cache: c.clone(),
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
        let handler_interaction_command_ping_str = r#"
            {
                "id": "0",
                "application_id": "0",
                "type": 2,
                "data": {
                    "id": "0",
                    "name": "ping",
                    "type": 255,
                    "resolved": {},
                    "options": [],
                    "target_id": null
                },
                "channel_id": "0",
                "user": {
                    "id": "0",
                    "avatar": null,
                    "bot": false,
                    "discriminator": "0000",
                    "username": "",
                    "public_flags": null,
                    "banner": null,
                    "accent_color": null
                },
                "token": "",
                "version": 0,
                "app_permissions": "104320065",
                "locale": "en-US",
                "guild_locale": "en-US"
            }
        "#;
        let handler_interaction_command_id_str = r#"
            {
                "id": "0",
                "application_id": "0",
                "type": 2,
                "data": {
                    "id": "0",
                    "name": "id",
                    "type": 255,
                    "resolved": {
                        "users": {
                            "418980020498009988": {
                                "id": "418980020498009988",
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
                            "value": "418980020498009988",
                            "type": 6,
                            "options": [],
                            "resolved": {
                                "User": [
                                    {
                                        "id":"418980020498009988",
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
                "channel_id": "0",
                "user": {
                    "id": "0",
                    "avatar": null,
                    "bot": false,
                    "discriminator": "0000",
                    "username": "",
                    "public_flags": null,
                    "banner": null,
                    "accent_color": null
                },
                "token": "",
                "version": 0,
                "app_permissions": "104320065",
                "locale": "en-US",
                "guild_locale": "en-US"
            }
        "#;
        //ping
        let handler_interaction_command_ping: CommandInteraction =
            from_str(handler_interaction_command_ping_str).unwrap();
        let handler_interaction_ping = Interaction::Command(handler_interaction_command_ping);
        let _ = handler.interaction_create(handler_context.clone(), handler_interaction_ping).await;
        //id
        let handler_interaction_command_id: CommandInteraction =
            from_str(handler_interaction_command_id_str).unwrap();
        dbg!(&handler_interaction_command_id);
        let handler_interaction_id = Interaction::Command(handler_interaction_command_id);
        dbg!(&handler_interaction_id);
        let _ = handler.interaction_create(handler_context, handler_interaction_id).await;
    }

    #[tokio::test]
    #[should_panic]
    async fn handler_interaction_create_unimplemented() {
        let cache = Cache::new();
        let c = Arc::new(cache);
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
            cache: c.clone(),
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
        let handler_interaction_command_never_str = r#"
            {
                "id": "0",
                "application_id": "0",
                "type": 2,
                "data": {
                    "id": "0",
                    "name": "💀",
                    "type": 255,
                    "resolved": {
                        "users": {
                            "418980020498009988": {
                                "id": "418980020498009988",
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
                "channel_id": "0",
                "user": {
                    "id": "0",
                    "avatar": null,
                    "bot": false,
                    "discriminator": "0000",
                    "username": "",
                    "public_flags": null,
                    "banner": null,
                    "accent_color": null
                },
                "token": "",
                "version": 0,
                "app_permissions": "104320065",
                "locale": "en-US",
                "guild_locale": "en-US"
            }
        "#;
        //unimplemented
        let handler_interaction_command_never: CommandInteraction =
            from_str(handler_interaction_command_never_str).unwrap();
        let handler_interaction_never = Interaction::Command(handler_interaction_command_never);
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
        let cache = Cache::new();
        let c = Arc::new(cache);
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
            cache: c.clone(),
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
        let message_str = r#"{
            "id": "1093709276008161320",
            "attachments": [],
            "author": {
                "id": "418980020498009988",
                "avatar": "d41d8cd98f00b204e9800998ecf8427e",
                "bot": true,
                "discriminator": "0000",
                "username": "Test",
                "public_flags": null,
                "banner": null,
                "accent_color": null
            },
            "channel_id": "0",
            "content": "Test content",
            "edited_timestamp": null,
            "embeds": [],
            "guild_id": "0",
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
