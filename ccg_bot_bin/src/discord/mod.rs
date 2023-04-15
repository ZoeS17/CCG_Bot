//!This way be Discord

//crate
use crate::config::Config;
use crate::utils::commandinteraction::CommandInteraction;

//serenity
use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::prelude::application::command::CommandOptionType;
use serenity::model::prelude::*;
use serenity::prelude::*;

//std
use std::error;
use std::fmt;

//re-exports
#[cfg(all(any(feature = "discord", feature = "full"), not(test)))]
mod builders;
#[cfg(all(any(feature = "discord", feature = "full"), test))]
pub mod builders;

#[doc(hidden)]
mod cache;
#[cfg(all(any(feature = "discord", feature = "full"), not(test)))]
mod commands;
#[cfg(all(any(feature = "discord", feature = "full"), test))]
pub mod commands;

#[derive(Debug)]
pub struct Handler(pub Config);

#[cfg(test)]
#[derive(Debug)]
pub struct TestHandler {
    pub interaction: Interaction,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction.clone() {
            let context = ctx.clone();
            let cache = context.cache;
            trace!("{:?}", &command.data);
            let opt: CommandInteraction = match command.data.options.get(0) {
                Some(o) => (*o).clone().into(),
                None => CommandInteraction {
                    name: "".to_string(),
                    value: None,
                    kind: CommandOptionType::Unknown,
                    options: vec![],
                    resolved: None,
                    focused: false,
                },
            };
            debug!("{:?}", &opt);
            let content = match command.data.name.as_str() {
                "ping" => commands::ping::run(&opt, cache),
                "id" => commands::id::run(&opt, cache),
                _ => unimplemented!(),
            };

            if let Err(why) = command
                .create_interaction_response(&context.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.set_embed(content))
                })
                .await
            {
                error!("Cannot respond to slash command: {why}");
            }
        }
        // #[cfg(test)]
        // {
        //     let a = TestHandler{ interaction };
        //     println!("{:#?}", a);
        // }
    }

    async fn ready<'a>(&'a self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        let gid =
            GuildId(self.0.discord_guildid.clone().parse().expect("guildid must be an integer"));

        let commands = GuildId::set_application_commands(&gid, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::ping::register(command))
                .create_application_command(|command| commands::id::register(command))
        })
        .await;
        let mut vec_commands = Vec::new();
        let _ = commands.unwrap().drain(..).for_each(|c| vec_commands.push(c.name));
        info!("I now have the following guild slash commands: {:?}", vec_commands);
    }

    ///This prints every message the bot can see, in the format:
    ///<pre>[Channel] Author: Message</pre>
    async fn message<'a>(&'a self, ctx: Context, msg: Message) {
        let channel_name: String = match ctx.cache.guild_channel(msg.channel_id) {
            Some(channel) => channel.name,
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

    let intents: GatewayIntents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_PRESENCES;

    // mark these allows to not get a warning in tests::discord::it_works
    #[allow(unused_variables)]
    #[allow(unused_mut)]
    let mut client: Client = Client::builder(dt, intents)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::json::prelude::from_str;
    use crate::StdResult;
    use error::Error;
    use futures::channel::mpsc::unbounded;
    use serde::{Deserialize, Serialize};
    use serenity::{
        cache::Cache, client::bridge::gateway::ShardMessenger, http::Http,
        model::application::interaction::application_command::ApplicationCommandInteraction,
        model::channel::ChannelType as SerenityChannelType,
        model::channel::GuildChannel as SerenityGuildChannel,
        model::event::ChannelUpdateEvent as SerenityChannelUpdateEvent,
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use typemap_rev::TypeMap;

    #[tokio::test]
    async fn handler_interaction_create() {
        let sender = unbounded().0;
        let handler_context = Context {
            data: Arc::new(RwLock::new(TypeMap::new())),
            shard: ShardMessenger::new(sender),
            shard_id: 0_64,
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
        let handler_interaction_command_ping: ApplicationCommandInteraction =
            from_str(handler_interaction_command_ping_str).unwrap();
        let handler_interaction_ping =
            Interaction::ApplicationCommand(handler_interaction_command_ping);
        let _ = handler.interaction_create(handler_context.clone(), handler_interaction_ping).await;
        //id
        let handler_interaction_command_id: ApplicationCommandInteraction =
            from_str(handler_interaction_command_id_str).unwrap();
        dbg!(&handler_interaction_command_id);
        let handler_interaction_id =
            Interaction::ApplicationCommand(handler_interaction_command_id);
        dbg!(&handler_interaction_id);
        let _ = handler.interaction_create(handler_context, handler_interaction_id).await;
    }

    #[tokio::test]
    #[should_panic]
    async fn handler_interaction_create_unimplemented() {
        let sender = unbounded().0;
        let handler_context = Context {
            data: Arc::new(RwLock::new(TypeMap::new())),
            shard: ShardMessenger::new(sender),
            shard_id: 0_64,
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
        let handler_interaction_command_never: ApplicationCommandInteraction =
            from_str(handler_interaction_command_never_str).unwrap();
        let handler_interaction_never =
            Interaction::ApplicationCommand(handler_interaction_command_never);
        let _ = handler.interaction_create(handler_context, handler_interaction_never).await;
    }

    fn message_count_patch<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> StdResult<Option<u8>, D::Error> {
        let real_count = Option::<u32>::deserialize(deserializer)?;
        Ok(real_count.map(u8::try_from).transpose().unwrap_or(Some(u8::MAX)))
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
        Unknown = !0,
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
                SerenityChannelType::Unknown => ChannelType::Unknown,
                _ => unimplemented!("Unknown type {value:?}"),
            };
            chantype
        }
    }

    // #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct GuildChannel {
        pub id: ChannelId,
        pub bitrate: Option<u64>,
        pub parent_id: Option<ChannelId>,
        pub guild_id: GuildId,
        #[serde(rename = "type")]
        pub kind: ChannelType,
        pub last_message_id: Option<MessageId>,
        pub last_pin_timestamp: Option<Timestamp>,
        pub name: String,
        #[serde(default)]
        pub permission_overwrites: Vec<PermissionOverwrite>,
        #[serde(default)]
        pub position: i64,
        pub topic: Option<String>,
        pub user_limit: Option<u64>,
        #[serde(default)]
        pub nsfw: bool,
        #[serde(default)]
        pub rate_limit_per_user: Option<u64>,
        pub rtc_region: Option<String>,
        pub video_quality_mode: Option<VideoQualityMode>,
        #[serde(default, deserialize_with = "message_count_patch")]
        pub message_count: Option<u8>,
        pub member_count: Option<u8>,
        pub thread_metadata: Option<ThreadMetadata>,
        pub member: Option<ThreadMember>,
        pub default_auto_archive_duration: Option<u64>,
    }

    impl From<GuildChannel> for SerenityGuildChannel {
        fn from(_value: GuildChannel) -> SerenityGuildChannel {
            GuildChannel::default().into()
        }
    }

    impl Default for GuildChannel {
        fn default() -> Self {
            Self {
                id: Default::default(),
                bitrate: Default::default(),
                parent_id: Default::default(),
                guild_id: Default::default(),
                kind: ChannelType::default(),
                last_message_id: Default::default(),
                last_pin_timestamp: Default::default(),
                name: Default::default(),
                permission_overwrites: Default::default(),
                position: Default::default(),
                topic: Default::default(),
                user_limit: Default::default(),
                nsfw: Default::default(),
                rate_limit_per_user: Default::default(),
                rtc_region: Default::default(),
                video_quality_mode: Default::default(),
                message_count: Default::default(),
                member_count: Default::default(),
                thread_metadata: Default::default(),
                member: Default::default(),
                default_auto_archive_duration: Default::default(),
            }
        }
    }

    impl From<SerenityGuildChannel> for GuildChannel {
        fn from(value: SerenityGuildChannel) -> Self {
            Self {
                id: value.id,
                bitrate: value.bitrate,
                parent_id: value.parent_id,
                guild_id: value.guild_id,
                kind: value.kind.into(),
                last_message_id: value.last_message_id,
                last_pin_timestamp: value.last_pin_timestamp,
                name: value.name,
                permission_overwrites: value.permission_overwrites,
                position: value.position,
                topic: value.topic,
                user_limit: value.user_limit,
                nsfw: value.nsfw,
                rate_limit_per_user: value.rate_limit_per_user,
                rtc_region: value.rtc_region,
                video_quality_mode: value.video_quality_mode,
                message_count: value.message_count,
                member_count: value.member_count,
                thread_metadata: value.thread_metadata,
                member: value.member,
                default_auto_archive_duration: value.default_auto_archive_duration,
            }
        }
    }

    pub(crate) struct ChannelUpdateEvent {
        channel: Channel,
    }

    impl From<SerenityChannelUpdateEvent> for ChannelUpdateEvent {
        fn from(value: SerenityChannelUpdateEvent) -> Self {
            Self { channel: value.channel }
        }
    }

    impl From<ChannelUpdateEvent> for SerenityChannelUpdateEvent {
        fn from(value: ChannelUpdateEvent) -> SerenityChannelUpdateEvent {
            ChannelUpdateEvent { channel: value.channel }.into()
        }
    }

    #[tokio::test]
    async fn handler_ready() {
        let sender = unbounded().0;
        let handler_context = Context {
            data: Arc::new(RwLock::new(TypeMap::new())),
            shard: ShardMessenger::new(sender),
            shard_id: 0_64,
            http: Arc::new(Http::new("")),
            cache: Arc::new(Cache::new()),
        };
        let mut e: SerenityChannelUpdateEvent =
            ChannelUpdateEvent { channel: Channel::Guild(GuildChannel::default().into()) }.into();
        handler_context.cache.update(&mut e);
        let handler = Handler(Config::default());
        let ready_str = r#"{
            "application": {
                "id": "0",
                "flags": 565248
            },
            "guilds": [
                {
                    "id": "0",
                    "unavailable": true
                }
            ],
            "presences": [],
            "private_channels": [],
            "session_id": "d41d8cd98f00b204e9800998ecf8427e",
            "shard": [
                0,
                1
            ],
            "_trace": [
                "[\"gateway-prd-us-east1-d-1mp8\",{\"micros\":116275,\"calls\":[\"id_created\",{\"micros\":933,\"calls\":[]},\"session_lookup_time\",{\"micros\":9743,\"calls\":[]},\"session_lookup_finished\",{\"micros\":17,\"calls\":[]},\"discord-sessions-blue-prd-2-165\",{\"micros\":104875,\"calls\":[\"start_session\",{\"micros\":52231,\"calls\":[\"discord-api-5bf757bbc6-dqbm2\",{\"micros\":47627,\"calls\":[\"get_user\",{\"micros\":16147},\"get_guilds\",{\"micros\":4372},\"send_scheduled_deletion_message\",{\"micros\":11},\"guild_join_requests\",{\"micros\":2},\"authorized_ip_coro\",{\"micros\":9}]}]},\"starting_guild_connect\",{\"micros\":73,\"calls\":[]},\"presence_started\",{\"micros\":10974,\"calls\":[]},\"guilds_started\",{\"micros\":106,\"calls\":[]},\"guilds_connect\",{\"micros\":2,\"calls\":[]},\"presence_connect\",{\"micros\":41445,\"calls\":[]},\"connect_finished\",{\"micros\":41450,\"calls\":[]},\"build_ready\",{\"micros\":18,\"calls\":[]},\"clean_ready\",{\"micros\":21,\"calls\":[]},\"optimize_ready\",{\"micros\":0,\"calls\":[]},\"split_ready\",{\"micros\":1,\"calls\":[]}]}]}]"
            ],
            "user": {
                "id": "418980020498009988",
                "avatar": "d41d8cd98f00b204e9800998ecf8427e",
                "bot": true,
                "discriminator": "0000",
                "email": null,
                "mfa_enabled": true,
                "username": "Test",
                "verified": true,
                "public_flags": null,
                "banner": null,
                "accent_colour": null
            },
            "v": 10
        }"#;
        let ready = from_str(ready_str).unwrap();
        let _ = handler.ready(handler_context, ready).await;
    }

    #[tokio::test]
    async fn handler_message() {
        let sender = unbounded().0;
        let handler_context = Context {
            data: Arc::new(RwLock::new(TypeMap::new())),
            shard: ShardMessenger::new(sender),
            shard_id: 0_64,
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
