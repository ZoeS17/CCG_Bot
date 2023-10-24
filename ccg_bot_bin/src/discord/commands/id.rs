//!Returns an embed with with user id, name, a mention,
//!, the user's avatar, and (if in the guild) a list of roles.

//crate imports
use crate::discord::builders::discordembed::*;
#[cfg(any(feature = "discord", feature = "full"))]
use crate::utils::commandinteraction::{CommandInteraction, CommandInteractionResolved};

//serenity imports
use serenity::all::CommandOptionType;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedAuthor};
use serenity::model::guild::PartialMember;
use serenity::model::Color;
use serenity::prelude::Context;

//std imports
use std::sync::Arc;

///Called when the command is run in a guild.
pub async fn run(options: &CommandInteraction, context: &Context) -> CreateEmbed {
    debug!("{:?}", options.clone());
    let c = &*Arc::try_unwrap(context.cache.clone()).unwrap_err();
    let http_cache = context.http.clone();
    let current_user = context.cache.current_user().clone();
    let option = options.resolved.as_ref().expect("Expected user object");

    let res: CreateEmbed;
    let CommandInteractionResolved::User(uid) = option else {
        panic!("unexpected type in resolved")
    };
    let user = uid.to_user(http_cache).await.expect("");
    let member = user.member.clone();
    let mut mem: PartialMember;
    if member.is_some() {
        mem = *member.clone().unwrap();
        //This is cursed. There has to be a better way.
        let mut roles = format!(
            "{:?}",
            mem.roles
                .drain(..)
                .map(|r| format!("{}", r.to_role_cached(c).unwrap()))
                .collect::<Vec<_>>()
        );
        roles.retain(|c| c != '[');
        roles.retain(|c| c != ']');
        roles.retain(|c| c != '"');
        let embed = DiscordEmbed::new()
            .field("id", format!("`{}`", user.id), true)
            .field("name", format!("`{}`", user.name), true)
            .field("mention", format!("<@{}>", user.id), true)
            .field("roles", roles, false)
            .thumbnail(user.face())
            .color(Color::new(0x500060_u32))
            .title(format!("{}'s info (w/ guild roles)", user.name))
            .author(CreateEmbedAuthor::new(current_user.name.to_string()).url(current_user.face()))
            .build();
        debug!("{:?}", &embed);
        debug!("{:?}", &mem);
        res = embed;
    } else {
        let embed = DiscordEmbed::new()
            .field("id", format!("`{}`", user.id), true)
            .field("name", format!("`{}`", user.name), true)
            .field("mention", format!("<@{}>", user.id), true)
            .thumbnail(user.face())
            .color(Color::new(0x500060_u32))
            .title(format!("{}'s info", user.name))
            .author(CreateEmbedAuthor::new(current_user.name.to_string()).url(current_user.face()))
            .build();
        debug!("{:?}", &embed);
        res = embed;
    }
    res
}

///Register the command to be used in the guild.
pub fn register() -> CreateCommand {
    CreateCommand::new("id").description("Get a user id").add_option(
        // CreateCommandOption(type, name, description)
        CreateCommandOption::new(CommandOptionType::User, "id", "The user to lookup")
            .required(true),
    )
}
