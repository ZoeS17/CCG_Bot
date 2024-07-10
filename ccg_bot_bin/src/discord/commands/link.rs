//!Link Discord and Twitch accounts from a discord command interaction

//crate imports
use crate::discord::builders::discordembed::*;
//skip reordering to allow easy reference to verbosity(from least to most)
#[rustfmt::skip]
use crate::debug;
use crate::utils::commandinteraction::CommandInteraction;
#[cfg(test)]
use crate::utils::commandinteraction::CommandInteractionResolved;

//serenity imports
use serenity::all::CommandOptionType;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedAuthor};
use serenity::model::Color;
use serenity::prelude::Context;

//std imports
// use std::sync::Arc;

///Called when the command is run in a guild.
pub async fn run(options: &CommandInteraction, context: &Context) -> CreateEmbed {
    debug!("{:?}", options.clone());
    // let c = &*Arc::try_unwrap(context.cache.clone()).unwrap_err();
    // let http_cache = context.http.clone();
    let current_user = context.cache.current_user().clone();
    #[cfg(test)]
    dbg!(&context.clone().http);
    #[cfg(test)]
    dbg!(&context.clone().cache);
    #[cfg(test)]
    let option: CommandInteractionResolved =
        options.data.options.first().expect("").value.clone().into();
    #[cfg(test)]
    dbg!(&option);

    // let res: CreateEmbed;
    let embed = DiscordEmbed::new()
        // .field("Twitch", format!("", ), false)
        .field("Twitch", "pending", false)
        // .field("Discord", format!("", ), false)
        .field("Discord", "impl needed", false)
        .color(Color::new(0x500060_u32))
        .title("Username linking")
        .author(CreateEmbedAuthor::new(current_user.name.to_string()).url(current_user.face()))
        .build();
    embed
}

///Register the command to be used in the guild.
pub fn register() -> CreateCommand {
    CreateCommand::new("link")
        .description("Link a Discord and twitch user")
        .add_option(
            // CreateCommandOption(type, name, description)
            CreateCommandOption::new(CommandOptionType::User, "discord", "Discord ID to link")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "twitch",
                "Twitch username to link",
            )
            .required(true),
        )
}
