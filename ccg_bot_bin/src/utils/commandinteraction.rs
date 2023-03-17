//!Reimplimentation of some Serenity's [application_command] structs and enums as they were non_exhaustive.
//!
//! [application_command]: serenity::model::application::interaction::application_command

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

///Reimplimentation of Serenity's [CommandType] as it was non_exhaustive
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

///Reimplimentation of Serenity's [CommandDataOptionValue] as it was non_exhaustive
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

///Reimplimentation of Serenity's [CommandDataOption] as it was non_exhaustive
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

#[cfg(test)]
mod tests {
    use crate::tests::discord::TestUser;
    use serenity::model::prelude::command::{CommandOptionType, CommandType};
    use serenity::model::user::User;
    /*use serenity::model::{
        channel::{Attachment, PartialChannel},
        guild::Role,
        user::User,
    };*/
    use super::*;
    use std::hash::Hash;

    #[test]
    fn derives_on_localcommandtype() {
        let upstream = LocalCommandType::ChatInput;
        let copy = upstream;
        let clone = LocalCommandType::ChatInput.clone();
        assert_eq!(copy, clone);
        assert!(upstream < LocalCommandType::User);
        let _ = upstream.hash(&mut std::collections::hash_map::DefaultHasher::new());
    }

    #[test]
    fn impl_from_commandtype_for_localcommandtype() {
        let upstream: CommandType = CommandType::User;
        let _: LocalCommandType = LocalCommandType::from(upstream);
    }

    #[test]
    fn derives_on_commandinteractionresolved() {
        let cir = CommandInteractionResolved::String("Test".to_string());
        let _ = CommandInteractionResolved::String("Test".to_string()).clone(); //derive(Clone)
        let _ = format!("{:?}", cir); //derive(Debug)
        let _ = serde_json::to_string(&cir); //derive(Serialize)
    }

    #[test]
    fn impl_from_commanddataoptionvalue_for_commandinteractionresolved() {
        let user = from_str::<User>(&to_string(&TestUser::default()).unwrap()).unwrap();
        let upstream_user: CommandDataOptionValue = CommandDataOptionValue::User(user, None);
        let upstream_string: CommandDataOptionValue =
            CommandDataOptionValue::String("Test".to_string());
        let upstream_int: CommandDataOptionValue = CommandDataOptionValue::Integer(1_i64);
        let upstream_bool: CommandDataOptionValue = CommandDataOptionValue::Boolean(false);
        let upstream_num: CommandDataOptionValue = CommandDataOptionValue::Number(1.0_f64);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_user);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_string);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_int);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_bool);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_num);
        //FIXME: test Channel(PartialChanel), Role(r), and Attachment(a) variants of
        //       CommandInteractionValue
    }

    #[test]
    fn derives_on_commandinteraction() {
        let test_resolved =
            CommandInteractionResolved::from(CommandDataOptionValue::String("Test".to_string()));
        let test_interaction: CommandInteraction = CommandInteraction {
            name: "".to_string(),
            value: None,
            kind: CommandOptionType::String,
            options: vec![],
            resolved: Some(test_resolved),
            focused: false,
        };
        let _ = test_interaction.clone(); //derive(Clone)
        let ti_string = serde_json::to_string(&test_interaction); //derive(Serialize)
        let _ = serde_json::from_str::<CommandInteraction>(&ti_string.unwrap()); //impl Deserialize
        let _ = format!("{:?}", test_interaction); //derive(Debug)
    }

    #[test]
    fn impl_from_commanddataoption_for_commandinteraction() {
        let test_ci = CommandInteraction {
            name: "".to_string(),
            value: None,
            kind: CommandOptionType::String,
            options: vec![],
            resolved: None,
            focused: false,
        };
        let test_cdo: CommandDataOption =
            serde_json::from_str::<CommandDataOption>(&serde_json::to_string(&test_ci).unwrap())
                .unwrap();
        let _: CommandInteraction = CommandInteraction::from(test_cdo);
    }

    #[test]
    fn enum_number_macro() {
        let mesg_num = LocalCommandType::Message.num();
        let user_num = LocalCommandType::User.num();
        let chat_num = LocalCommandType::ChatInput.num();
        assert_eq!(mesg_num, 3_u64);
        assert_eq!(user_num, 2_u64);
        assert_eq!(chat_num, 1_u64);
        let msg = serde_json::to_string(&LocalCommandType::Message).unwrap();
        assert_eq!(msg, "3".to_string());
        let lct = serde_json::from_str::<LocalCommandType>(&msg).unwrap();
        assert_eq!(lct, LocalCommandType::Message);
    }
}
