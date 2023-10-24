//!Reimplimentation of some Serenity's [CommandInteraction] structs and enums as they were non_exhaustive.
//!
//! [CommandInteraction]: serenity::model::application::CommandInteraction

//crate
use crate::StdResult;
//serde
use serde::ser::{Error as SerError, SerializeStructVariant};
use serde::{Deserialize, Serialize};

//serenity
use serenity::all::{
    AttachmentId, ChannelId, Color, CommandOptionType, CommandType, RoleId, UserId,
};
use serenity::model::{
    application::{CommandDataOption, CommandDataOptionValue},
    channel::ChannelType,
    channel::{Attachment as SerenityAttachment, PartialChannel as SerenityPartialChannel},
    guild::{Role as SerenityRole, RoleTags as SerenityRoleTags},
    Permissions,
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
            CommandType::Unknown(_) => LocalCommandType::Unknown,
            _ => unimplemented!(),
        }
    }
}

///Reimplimentation of Serenity's [CommandDataOptionValue] as it was non_exhaustive
#[derive(Clone, Debug, Deserialize)]
pub enum CommandInteractionResolved {
    String(String),
    Integer(i64),
    Boolean(bool),
    User(UserId),
    Channel(ChannelId),
    Role(RoleId),
    Number(f64),
    Attachment(AttachmentId),
}

impl From<CommandDataOptionValue> for CommandInteractionResolved {
    fn from(cdov: CommandDataOptionValue) -> CommandInteractionResolved {
        match cdov {
            CommandDataOptionValue::String(s) => CommandInteractionResolved::String(s),
            CommandDataOptionValue::Integer(i) => CommandInteractionResolved::Integer(i),
            CommandDataOptionValue::Boolean(b) => CommandInteractionResolved::Boolean(b),
            CommandDataOptionValue::User(uid) => CommandInteractionResolved::User(uid),
            CommandDataOptionValue::Channel(pc) => CommandInteractionResolved::Channel(pc),
            CommandDataOptionValue::Role(r) => CommandInteractionResolved::Role(r),
            CommandDataOptionValue::Number(f) => CommandInteractionResolved::Number(f),
            CommandDataOptionValue::Attachment(a) => CommandInteractionResolved::Attachment(a),
            _ => unimplemented!(),
        }
    }
}

///Reimplimentation of Serenity's [CommandDataOption] as it was non_exhaustive
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommandInteraction {
    /// The name of the parameter.
    pub name: String,
    /// The given value.
    #[serde(serialize_with = "serialize_cdov", skip_deserializing, default = "default_cdov")]
    pub value: CommandDataOptionValue,
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

impl std::fmt::Display for CommandInteraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {:?}, {:?}, {:?}, {:?}, {}",
            self.name, self.value, self.kind, self.options, self.resolved, self.focused
        )
    }
}

pub(crate) fn default_cdov() -> CommandDataOptionValue {
    CommandDataOptionValue::Unknown(!0u8)
}

/*
impl<'de> Deserialize<'de> for CommandInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let name = map
            .remove("name")
            .ok_or_else(|| DeError::custom("expected String"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;

        let value = map
            .remove("value")
            .ok_or_else(|| DeError::custom("expected CommandOptionValue variant"))
            .and_then(CommandInteractionResolved::deserialize)
            .and_then(|s| serde_json::to_string(&s))
            .and_then(|l| serde_json::from_str(&l))
            .map_err(DeError::custom)?;

        debug!("{:?}", &value);

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

        Ok(Self {
            name,
            value: CommandDataOptionValue::Unknown(!0u8),
            kind,
            options,
            resolved: None,
            focused,
        })
    }
}
*/

impl Serialize for CommandInteractionResolved {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            CommandInteractionResolved::String(s) => serializer.serialize_str(s.as_str()),
            CommandInteractionResolved::Integer(i) => serializer.serialize_i64(*i),
            CommandInteractionResolved::Boolean(b) => serializer.serialize_bool(*b),
            // Since serenity uses a NonZeroU64 instead of a standard u64
            CommandInteractionResolved::User(u) => serializer.serialize_u64(u.get()),
            // Since serenity uses a NonZeroU64 instead of a standard u64
            CommandInteractionResolved::Channel(c) => serializer.serialize_u64(c.get()),
            // Since serenity uses a NonZeroU64 instead of a standard u64
            CommandInteractionResolved::Role(r) => serializer.serialize_u64(r.get()),
            CommandInteractionResolved::Number(n) => serializer.serialize_f64(*n),
            // Since serenity uses a NonZeroU64 instead of a standard u64
            CommandInteractionResolved::Attachment(a) => serializer.serialize_u64(a.get()),
        }
    }
}

pub(crate) fn serialize_cdov<S>(
    cdov: &CommandDataOptionValue,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match *cdov {
        CommandDataOptionValue::Autocomplete { ref kind, ref value } => {
            let mut state = serializer.serialize_struct_variant(
                "CommandDataOptionValue",
                0u32,
                "Autocomplete",
                2,
            )?;
            state.serialize_field("kind", kind)?;
            state.serialize_field("value", value)?;
            state.end()
        },
        CommandDataOptionValue::Boolean(ref b) => {
            serializer.serialize_newtype_variant("CommandDataOptionValue", 1u32, "Boolean", b)
        },
        CommandDataOptionValue::Integer(ref i) => {
            serializer.serialize_newtype_variant("CommandDataOptionValue", 2u32, "Integer", i)
        },
        CommandDataOptionValue::Number(ref n) => {
            serializer.serialize_newtype_variant("CommandDataOptionValue", 3u32, "Number", n)
        },
        CommandDataOptionValue::String(ref s) => {
            serializer.serialize_newtype_variant("CommandDataOptionValue", 4u32, "String", s)
        },
        CommandDataOptionValue::SubCommand(ref sc) => {
            serializer.serialize_newtype_variant("CommandDataOptionValue", 5u32, "SubCommand", sc)
        },
        CommandDataOptionValue::SubCommandGroup(ref scg) => serializer.serialize_newtype_variant(
            "CommandDataOptionValue",
            6u32,
            "SubCommandGroup",
            scg,
        ),
        CommandDataOptionValue::Attachment(ref a) => {
            serializer.serialize_newtype_variant("CommandDataOptionValue", 7u32, "Attachment", a)
        },
        CommandDataOptionValue::Channel(ref c) => {
            serializer.serialize_newtype_variant("CommandDataOptionValue", 8u32, "Channel", c)
        },
        CommandDataOptionValue::Mentionable(ref m) => {
            serializer.serialize_newtype_variant("CommandDataOptionValue", 9u32, "Mentionable", m)
        },
        CommandDataOptionValue::Role(ref r) => {
            serializer.serialize_newtype_variant("CommandDataOptionValue", 10u32, "Role", r)
        },
        CommandDataOptionValue::User(ref u) => {
            serializer.serialize_newtype_variant("CommandDataOptionValue", 11u32, "User", u)
        },
        CommandDataOptionValue::Unknown(ref ukn) => {
            serializer.serialize_newtype_variant("CommandDataOptionValue", 12u32, "Unknown", ukn)
        },
        _ => Err(SerError::custom("Unable to serialize CommandDataOptionValue")),
    }
}

pub(crate) mod snowflake {
    use serde::de::{Error, Visitor};
    use serde::{Deserializer, Serializer};
    use std::convert::TryFrom;
    use std::fmt;
    use std::num::NonZeroU64;
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NonZeroU64, D::Error> {
        deserializer.deserialize_any(SnowflakeVisitor)
    }
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: Serializer>(id: &NonZeroU64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&id.get())
    }
    struct SnowflakeVisitor;
    impl<'de> Visitor<'de> for SnowflakeVisitor {
        type Value = NonZeroU64;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a non-zero string or integer snowflake")
        }
        fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
            self.visit_u64(u64::try_from(value).map_err(Error::custom)?)
        }
        fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
            NonZeroU64::new(value).ok_or_else(|| Error::custom("invalid value, expected non-zero"))
        }
        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            value.parse().map_err(Error::custom)
        }
    }
}

impl From<CommandDataOption> for CommandInteraction {
    fn from(cdo: CommandDataOption) -> CommandInteraction {
        Self {
            name: cdo.name.clone(),
            value: cdo.value.clone(),
            kind: cdo.kind(),
            options: vec![],
            resolved: None,
            focused: false,
        }
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_false(v: &bool) -> bool {
    !v
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Attachment {
    pub id: serenity::model::id::AttachmentId,
    pub filename: String,
    pub height: Option<u32>,
    pub proxy_url: String,
    pub size: u32,
    pub url: String,
    pub width: Option<u32>,
    pub content_type: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ephemeral: bool,
}

impl From<SerenityAttachment> for Attachment {
    fn from(value: SerenityAttachment) -> Self {
        Self {
            id: value.id,
            filename: value.filename,
            height: value.height,
            proxy_url: value.proxy_url,
            size: value.size,
            url: value.url,
            width: value.width,
            content_type: value.content_type,
            ephemeral: value.ephemeral,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Role {
    pub id: serenity::model::id::RoleId,
    pub guild_id: serenity::model::id::GuildId,
    #[serde(rename = "color")]
    pub colour: Color,
    pub hoist: bool,
    pub managed: bool,
    #[serde(default)]
    pub mentionable: bool,
    pub name: String,
    #[serde(default)]
    pub permissions: Permissions,
    pub position: u16,
    #[serde(default)]
    pub tags: RoleTags,
    pub icon: Option<String>,
    pub unicode_emoji: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct RoleTags {
    pub bot_id: Option<serenity::model::id::UserId>,
    pub integration_id: Option<serenity::model::id::IntegrationId>,
    #[serde(default, skip_serializing_if = "is_false", with = "premium_subscriber")]
    pub premium_subscriber: bool,
}

impl From<SerenityRoleTags> for RoleTags {
    fn from(value: SerenityRoleTags) -> Self {
        Self {
            bot_id: value.bot_id,
            integration_id: value.integration_id,
            premium_subscriber: value.premium_subscriber,
        }
    }
}

// A premium subscriber role is reported with the field present and the value `null`.
mod premium_subscriber {
    use std::fmt;

    use serde::de::{Error, Visitor};
    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
        deserializer.deserialize_option(NullValueVisitor)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S: Serializer>(_: &bool, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_none()
    }

    struct NullValueVisitor;

    impl<'de> Visitor<'de> for NullValueVisitor {
        type Value = bool;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("null value")
        }

        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(true)
        }

        /// Called by the `simd_json` crate
        fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
            Ok(true)
        }
    }
}

impl From<SerenityRole> for Role {
    fn from(r: SerenityRole) -> Self {
        Self {
            id: r.id,
            guild_id: r.guild_id,
            colour: r.colour,
            hoist: r.hoist,
            managed: r.managed,
            mentionable: r.mentionable,
            name: r.name,
            permissions: r.permissions,
            position: r.position,
            tags: r.tags.into(),
            icon: None,
            unicode_emoji: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct PartialChannel {
    pub id: serenity::model::id::ChannelId,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub kind: ChannelType,
    pub permissions: Option<Permissions>,
}

impl Default for PartialChannel {
    fn default() -> Self {
        Self {
            id: serenity::model::id::ChannelId::default(),
            name: Some(String::default()),
            kind: ChannelType::Unknown(!0u8),
            permissions: Some(Permissions::default()),
        }
    }
}

impl From<SerenityPartialChannel> for PartialChannel {
    fn from(value: SerenityPartialChannel) -> Self {
        Self { id: value.id, name: value.name, kind: value.kind, permissions: value.permissions }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::discord::TestUser;
    use crate::utils::prelude::prelude::{from_str, to_string};
    use serenity::all::{CommandOptionType, User};
    use serenity::model::{
        channel::{Attachment as SerenityAttachment, PartialChannel as SerenityPartialChannel},
        guild::Role as SerenityRole,
    };
    use std::hash::Hash;

    mod command_interaction_serde {
        use super::CommandInteraction;
        use std::fmt;

        use serde::de::Visitor;
        use serde::{Deserializer, Serializer};

        pub fn deserialize<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<CommandInteraction, D::Error> {
            deserializer.deserialize_any(CommandInteractionVisitor)
        }

        #[allow(clippy::trivially_copy_pass_by_ref)]
        pub fn serialize<S: Serializer>(
            id: &CommandInteraction,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            serializer.collect_str(&id)
        }

        struct CommandInteractionVisitor;

        impl<'de> Visitor<'de> for CommandInteractionVisitor {
            type Value = CommandInteraction;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a valid CommandInteraction Struct")
            }

            //fn visit_() -> Result<Value, Error>{}
        }
    }

    #[test]
    fn derives_on_attachment() {
        let test_attach = Attachment::default(); // derive(Default)
        let _ = format!("{:?}", test_attach); // derive(Debug)
        let _ = test_attach.clone(); // derive(Clone)
        let test_attch_str = to_string(&test_attach).unwrap(); // derive(Serialize)
        let derived_attach = from_str::<Attachment>(&test_attch_str).unwrap(); // derive(Deserialize)
        assert_eq!(test_attach, derived_attach);
    }

    #[test]
    fn impl_from_serenityattachment_for_attachment() {
        let test_attach = Attachment::default();
        let test_attach_str = to_string(&test_attach).unwrap();
        let upstream_attach = from_str::<SerenityAttachment>(&test_attach_str).unwrap();
        let roundtrip = Attachment::from(upstream_attach);
        assert_eq!(test_attach, roundtrip);
    }

    #[test]
    fn derives_on_partial_channel() {
        let test_part_chan = PartialChannel::default(); //impl Default
        let _ = format!("{:?}", test_part_chan); // derive(Debug)
        let _ = test_part_chan.clone(); // derive(Clone)
        let test_pc_str = to_string(&test_part_chan).unwrap(); // derive(Serialize)
        let derived_pc = from_str::<PartialChannel>(&test_pc_str).unwrap(); // derive(Deserialize)
        assert_eq!(test_part_chan, derived_pc);
    }

    #[test]
    fn impl_from_serenitypartialchannel_for_partial_channel() {
        let test_partial_channel = PartialChannel::default();
        let test_partial_channel_str = to_string(&test_partial_channel).unwrap();
        let upstream_partial_channel =
            from_str::<SerenityPartialChannel>(&test_partial_channel_str).unwrap();
        let roundtrip = PartialChannel::from(upstream_partial_channel);
        assert_eq!(test_partial_channel, roundtrip);
    }

    #[test]
    fn derives_on_role() {
        let test_role = Role::default(); // derive(Default)
        let _ = format!("{:?}", test_role); // derive(Debug)
        let _ = test_role.clone(); // derive(Clone)
        let test_role_str = to_string(&test_role).unwrap(); // derive(Serialize)
        let derived_role = from_str::<Role>(&test_role_str).unwrap(); // derive(Deserialize)
        assert_eq!(test_role, derived_role);
    }

    #[test]
    fn impl_from_serenityrole_for_role() {
        let test_role = Role::default();
        let test_role_str = to_string(&test_role).unwrap();
        let upstream_role = from_str::<SerenityRole>(&test_role_str).unwrap();
        let roundtrip = Role::from(upstream_role);
        assert_eq!(test_role, roundtrip);
    }

    #[test]
    fn derives_on_roletags() {
        let test_role_tags = RoleTags::default(); // derive(Default)
        let _ = format!("{:?}", test_role_tags); // derive(Debug)
        let _ = test_role_tags.clone(); // derive(Clone)
        let test_roletags_str = to_string(&test_role_tags).unwrap(); // derive(Serialize)
        let derived_role_tags = from_str::<RoleTags>(&test_roletags_str).unwrap(); // derive(Deserialize)
        assert_eq!(test_role_tags, derived_role_tags);
    }

    #[test]
    fn impl_from_serenityroletags_for_roletags() {
        let test_role_tags = RoleTags::default();
        let test_roletags_str = to_string(&test_role_tags).unwrap();
        let upstream_roletags = from_str::<SerenityRoleTags>(&test_roletags_str).unwrap();
        let roundtrip = RoleTags::from(upstream_roletags);
        assert_eq!(test_role_tags, roundtrip);
    }

    #[test]
    fn derives_on_localcommandtype() {
        let upstream = LocalCommandType::ChatInput;
        let copy = upstream; // derive(Copy)
        let clone = LocalCommandType::ChatInput.clone(); // derive(Clone)
        assert_eq!(copy, clone);
        assert!(upstream < LocalCommandType::User); // derive(Eq, PartialEq, PartialOrd, Ord)
        upstream.hash(&mut std::collections::hash_map::DefaultHasher::new()); // derive(Hash)
        let _ = format!("{:?}", upstream); // derive(Debug)
    }

    #[test]
    fn impl_from_commandtype_for_localcommandtype() {
        let upstream: CommandType = CommandType::User;
        let _: LocalCommandType = LocalCommandType::from(upstream);
        let upstream: CommandType = CommandType::Message;
        let _: LocalCommandType = LocalCommandType::from(upstream);
        let upstream: CommandType = CommandType::ChatInput;
        let _: LocalCommandType = LocalCommandType::from(upstream);
        let upstream: CommandType = CommandType::Unknown(!0u8);
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
        let attach =
            from_str::<SerenityAttachment>(&to_string(&Attachment::default()).unwrap()).unwrap();
        let chan =
            from_str::<SerenityPartialChannel>(&to_string(&PartialChannel::default()).unwrap())
                .unwrap();
        let role = from_str::<SerenityRole>(&to_string(&Role::default()).unwrap()).unwrap();
        let upstream_user: CommandDataOptionValue = CommandDataOptionValue::User(user.id);
        let upstream_string: CommandDataOptionValue =
            CommandDataOptionValue::String("Test".to_string());
        let upstream_int: CommandDataOptionValue = CommandDataOptionValue::Integer(1_i64);
        let upstream_bool: CommandDataOptionValue = CommandDataOptionValue::Boolean(false);
        let upstream_num: CommandDataOptionValue = CommandDataOptionValue::Number(1.0_f64);
        let upstream_pc: CommandDataOptionValue = CommandDataOptionValue::Channel(chan.id);
        let upstream_attach: CommandDataOptionValue = CommandDataOptionValue::Attachment(attach.id);
        let upstream_role: CommandDataOptionValue = CommandDataOptionValue::Role(role.id);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_user);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_string);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_int);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_bool);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_num);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_pc);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_attach);
        let _: CommandInteractionResolved = CommandInteractionResolved::from(upstream_role);
    }

    #[test]
    fn derives_on_commandinteraction() {
        let test_resolved =
            CommandInteractionResolved::from(CommandDataOptionValue::String("Test".to_string()));
        let test_interaction: CommandInteraction = CommandInteraction {
            name: "".to_string(),
            value: CommandDataOptionValue::String(Default::default()),
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
            value: CommandDataOptionValue::String(Default::default()),
            kind: CommandOptionType::String,
            options: vec![],
            resolved: None,
            focused: false,
        };
        let test_cdo_str = serde_json::to_string(&test_ci).unwrap();
        let test_cdo: CommandDataOption =
            serde_json::from_str::<CommandDataOption>(&test_cdo_str).unwrap();
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
