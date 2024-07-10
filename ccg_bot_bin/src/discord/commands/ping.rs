//!Returns an embed with the message, "Program" in the Greetings field.

//crate
use crate::discord::builders::discordembed::*;
//skip reordering to allow easy reference to verbosity(from least to most)
#[rustfmt::skip]
use crate::debug;
// #[cfg(any(feature = "discord", feature = "full"))]
use crate::utils::commandinteraction::CommandInteraction;

//serenity
use serenity::all::{Color, Context};
use serenity::builder::{CreateCommand, CreateEmbed, CreateEmbedAuthor};

///Called when the command is run in a guild.
pub async fn run(_options: &CommandInteraction, context: &Context) -> CreateEmbed {
    let current_user = context.cache.current_user().clone();
    let embed = DiscordEmbed::new()
        .field("Greetings", "Program".to_string(), true)
        .color(Color::new(0x500060_u32))
        .thumbnail(
            "https://cdn.discordapp.com/emojis/938514423155400804.webp?size=48&quality=lossless",
        )
        .title("Pong")
        .author(CreateEmbedAuthor::new(current_user.name.to_string()).url(current_user.face()))
        .build();
    debug!("{:?}", &embed);
    embed
}

///Register the command to be used in the guild.
pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("A ping command")
}
