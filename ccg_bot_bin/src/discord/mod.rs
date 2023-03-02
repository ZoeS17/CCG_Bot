//!Discord module go brr!!!

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

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let context = ctx.clone();
            let cache = context.cache;
            trace!("{:?}", &command.data);
            //let cmd_opts = ;
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
        println!("[{}] {}: {}", channel_name, msg.author.name, msg.content);
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
    fn cause(&self) -> Option<&dyn error::Error> {
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
    let c = std::result::Result::Ok(Handler(Config {
        discord_guildid: "".to_string(),
        discord_token: "".to_string(),
        #[cfg(any(feature = "twitch", feature = "full"))]
        twitch_channels: vec!["".to_string()],
    }));

    c
}
