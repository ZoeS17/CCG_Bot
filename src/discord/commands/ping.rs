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

pub fn run(options: &CommandInteraction, cache: Arc<Cache>) -> CreateEmbed {
    let current_user = (*Arc::try_unwrap(cache).unwrap_err()).current_user();
    dbg!(options);
    let mut embed = DiscordEmbed::new()
        .field("Greetings", "Program".to_string(), true)
        .color(Color::new(0xCC00FF_u32))
        .build();
    embed.author(|a| a.name(current_user.name.to_string()).url(current_user.face()));
    embed
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("ping").description("A ping command")
}
