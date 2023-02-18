//crate
use crate::utils::json::prelude::*;
use crate::StdResult;
//serde
use serde::de::{Deserializer, Error as DeError};
use serde::{Deserialize, Serialize};

//serenity
use serenity::json::JsonMap;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::{
    application::interaction::application_command::{CommandDataOption, CommandDataOptionValue},
    channel::{Attachment, PartialChannel},
    guild::{PartialMember, Role},
    prelude::command::CommandType,
    user::User,
};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LocalCommandType {
    ChatInput = 1,
    User = 2,
    Message = 3,
    Unknown = !0,
}

enum_number!(LocalCommandType { ChatInput, User, Message });

impl From<CommandType> for LocalCommandType {
    fn from(ct: CommandType) -> LocalCommandType {
        match ct {
            CommandType::ChatInput => LocalCommandType::ChatInput,
            CommandType::User => LocalCommandType::User,
            CommandType::Message => LocalCommandType::Message,
            CommandType::Unknown => LocalCommandType::Unknown,
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum CommandInteractionResolved {
    String(String),
    Integer(i64),
    Boolean(bool),
    User(User, Option<PartialMember>),
    Channel(PartialChannel),
    Role(Role),
    Number(f64),
    Attachment(Attachment),
}

impl From<CommandDataOptionValue> for CommandInteractionResolved {
    fn from(cdov: CommandDataOptionValue) -> CommandInteractionResolved {
        match cdov {
            CommandDataOptionValue::String(s) => CommandInteractionResolved::String(s),
            CommandDataOptionValue::Integer(i) => CommandInteractionResolved::Integer(i),
            CommandDataOptionValue::Boolean(b) => CommandInteractionResolved::Boolean(b),
            CommandDataOptionValue::User(u, pm) => CommandInteractionResolved::User(u, pm),
            CommandDataOptionValue::Channel(pc) => CommandInteractionResolved::Channel(pc),
            CommandDataOptionValue::Role(r) => CommandInteractionResolved::Role(r),
            CommandDataOptionValue::Number(f) => CommandInteractionResolved::Number(f),
            CommandDataOptionValue::Attachment(a) => CommandInteractionResolved::Attachment(a),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandInteraction {
    /// The name of the parameter.
    pub name: String,
    /// The given value.
    pub value: Option<Value>,
    /// The value type.
    #[serde(rename = "type")]
    pub kind: CommandOptionType,
    /// The nested options.
    ///
    /// **Note**: It is only present if the option is
    /// a group or a subcommand.
    #[serde(default)]
    pub options: Vec<CommandInteraction>,
    /// The resolved object of the given `value`, if there is one.
    #[serde(default)]
    pub resolved: Option<CommandInteractionResolved>,
    /// For `Autocomplete` Interactions this will be `true` if
    /// this option is currently focused by the user.
    #[serde(default)]
    pub focused: bool,
}

impl<'de> Deserialize<'de> for CommandInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let name = map
            .remove("name")
            .ok_or_else(|| DeError::custom("expected value"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;

        let value = map.remove("value");

        let kind = map
            .remove("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(CommandOptionType::deserialize)
            .map_err(DeError::custom)?;

        let options = map
            .remove("options")
            .map(Vec::deserialize)
            .transpose()
            .map_err(DeError::custom)?
            .unwrap_or_default();

        let focused = match map.get("focused") {
            Some(value) => value.as_bool().ok_or_else(|| DeError::custom("expected bool"))?,
            None => false,
        };

        Ok(Self { name, value, kind, options, resolved: None, focused })
    }
}

impl From<CommandDataOption> for CommandInteraction {
    fn from(cdo: CommandDataOption) -> CommandInteraction {
        let opts: Vec<CommandInteraction> = cdo.options.into_iter().map(|o| o.into()).collect();
        let res: Option<CommandInteractionResolved> = cdo.resolved.map(|r| r.into());
        Self {
            name: cdo.name,
            value: cdo.value,
            kind: cdo.kind,
            options: opts,
            resolved: res,
            focused: cdo.focused,
        }
    }
}
