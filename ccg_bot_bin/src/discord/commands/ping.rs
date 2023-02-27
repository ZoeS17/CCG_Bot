//!Returns an embed with the message, "Program" in the Greetings field.

//crate
use crate::discord::builders::discordembed::*;
#[cfg(any(feature = "discord", feature = "full"))]
use crate::utils::commandinteraction::CommandInteraction;

//serenity
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::cache::Cache;
use serenity::utils::Color;

//std
use std::sync::Arc;

///Called when the command is run in a guild.
pub fn run(options: &CommandInteraction, cache: Arc<Cache>) -> CreateEmbed {
    let current_user = (*Arc::try_unwrap(cache).unwrap_err()).current_user();
    dbg!(options);
    let mut embed = DiscordEmbed::new()
        .field("Greetings", "Program".to_string(), true)
        .color(Color::new(0x500060_u32))
        .build();
    embed.author(|a| a.name(current_user.name.to_string()).url(current_user.face()));
    embed
}

///Register the command to be used in the guild.
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping").description("A ping command")
}
