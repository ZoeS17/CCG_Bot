//!Returns an embed with with user id, name, a mention,
//!, the user's avatar, and (if in the guild) a list of roles.

//crate imports
use crate::discord::builders::discordembed::*;
#[cfg(any(feature = "discord", feature = "full"))]
use crate::utils::commandinteraction::{CommandInteraction, CommandInteractionResolved};

//serenity imports
use serenity::builder::CreateApplicationCommand;
use serenity::builder::CreateEmbed;
use serenity::cache::Cache;
use serenity::model::guild::PartialMember;
use serenity::model::prelude::command::CommandOptionType;
use serenity::utils::Color;

//std imports
use std::sync::Arc;

///Called when the command is run in a guild.
pub fn run(options: &CommandInteraction, cache: Arc<Cache>) -> CreateEmbed {
    debug!("{:?}", options.clone());
    let c = &*Arc::try_unwrap(cache.clone()).unwrap_err();
    let current_user = (*Arc::try_unwrap(cache).unwrap_err()).current_user();
    let option = options.resolved.as_ref().expect("Expected user object");

    let res: CreateEmbed;
    let CommandInteractionResolved::User(user, member) = option else { panic!("unexpected type in resolved")};
    let mut mem: PartialMember;
    if member.is_some() {
        mem = member.clone().unwrap();
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
        let mut embed = DiscordEmbed::new()
            .field("id", format!("`{}`", user.id), true)
            .field("name", format!("`{}`", user.name), true)
            .field("mention", format!("<@{}>", user.id), true)
            .field("roles", roles, false)
            .thumbnail(user.face())
            .color(Color::new(0x500060_u32))
            .title(format!("{}'s info (w/ guild roles)", user.name))
            .build();
        embed.author(|a| a.name(current_user.name.to_string()).url(current_user.face()));
        debug!("{:?}", &embed);
        debug!("{:?}", &mem);
        res = embed;
    } else {
        let mut embed = DiscordEmbed::new()
            .field("id", format!("`{}`", user.id), true)
            .field("name", format!("`{}`", user.name), true)
            .field("mention", format!("<@{}>", user.id), true)
            .thumbnail(user.face())
            .color(Color::new(0x500060_u32))
            .title(format!("{}'s info", user.name))
            .build();
        embed.author(|a| a.name(current_user.name.to_string()).url(current_user.face()));
        debug!("{:?}", &embed);
        res = embed;
    }
    res
}

///Register the command to be used in the guild.
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("id").description("Get a user id").create_option(|option| {
        option
            .name("id")
            .description("The user to lookup")
            .kind(CommandOptionType::User)
            .required(true)
    })
}
