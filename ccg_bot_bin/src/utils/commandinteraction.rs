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
    ApplicationId, Attachment as SerenityAttachment, AttachmentId, AutoArchiveDuration, ChannelId,
    ChannelType, Color, CommandData, CommandDataOption, CommandDataOptionValue,
    CommandDataResolved, CommandId, CommandType, GuildId, Interaction, InteractionId, Member,
    PartialChannel as SerenityPartialChannel, Permissions, Role as SerenityRole, RoleId,
    RoleTags as SerenityRoleTags, TargetId, ThreadMetadata as SerenityThreadMetadata, User, UserId,
};
use serenity::model::Timestamp;

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
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The data of the interaction which was triggered.
    pub data: CommandData,
    /// The guild Id this interaction was sent from, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// Channel that the interaction was sent from.
    pub channel: Option<PartialChannel>,
    /// The channel Id this interaction was sent from.
    pub channel_id: ChannelId,
    /// The `member` data for the invoking user.
    ///
    /// **Note**: It is only present if the interaction is triggered in a guild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<Box<Member>>,
    /// The `user` object for the invoking user.
    #[serde(default)]
    pub user: User,
    /// A continuation token for responding to the interaction.
    pub token: String,
    /// Always `1`.
    pub version: u8,
    /// Permissions the app or bot has within the channel the interaction was sent from.
    pub app_permissions: Option<Permissions>,
    /// The selected language of the invoking user.
    pub locale: String,
    /// The guild's preferred locale.
    pub guild_locale: Option<String>,
}

impl std::fmt::Display for CommandInteraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {:?}, {:?}, {:?}, {:?}, {}, {:?}, {}, [REDACTED TOKEN], {}, {:?}, {}, {:?}",
            self.id,
            self.application_id,
            self.data,
            self.guild_id,
            self.channel,
            self.channel_id,
            self.member,
            self.user,
            self.version,
            self.app_permissions,
            self.locale,
            self.guild_locale,
        )
    }
}

pub(crate) fn default_cdov() -> CommandDataOptionValue {
    CommandDataOptionValue::Unknown(!0u8)
}

impl From<Interaction> for CommandInteraction {
    fn from(value: Interaction) -> Self {
        let ci = value.command().expect("Unable to get CommandInteraction from Interaction");
        Self {
            id: ci.id,
            application_id: ci.application_id,
            data: ci.data,
            guild_id: ci.guild_id,
            channel: Some(PartialChannel::from(ci.channel.expect(""))),
            channel_id: ci.channel_id,
            member: ci.member,
            user: ci.user,
            token: ci.token,
            version: ci.version,
            app_permissions: ci.app_permissions,
            locale: ci.locale,
            guild_locale: ci.guild_locale,
        }
    }
}

///Reimplimentation of Serenity's [CommandData] as it was non_exhaustive
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct LocalCommandData {
    /// The Id of the invoked command.
    pub id: CommandId,
    /// The name of the invoked command.
    pub name: String,
    /// The application command type of the triggered application command.
    #[serde(rename = "type")]
    pub kind: CommandType,
    /// The parameters and the given values. The converted objects from the given options.
    #[serde(default)]
    pub resolved: CommandDataResolved,
    #[serde(default)]
    pub options: Vec<CommandDataOption>,
    /// The Id of the guild the command is registered to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// The targeted user or message, if the triggered application command type is [`User`] or
    /// [`Message`].
    ///
    /// Its object data can be found in the [`resolved`] field.
    ///
    /// [`resolved`]: Self::resolved
    /// [`User`]: CommandType::User
    /// [`Message`]: CommandType::Message
    pub target_id: Option<TargetId>,
}

impl From<CommandData> for LocalCommandData {
    fn from(value: CommandData) -> Self {
        Self {
            id: value.id,
            name: value.name,
            kind: value.kind,
            resolved: value.resolved,
            options: value.options,
            guild_id: value.guild_id,
            target_id: value.target_id,
        }
    }
}

impl Default for LocalCommandData {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            kind: CommandType::Unknown(!0u8),
            resolved: Default::default(),
            options: Default::default(),
            guild_id: Default::default(),
            target_id: Default::default(),
        }
    }
}

impl Serialize for CommandInteractionResolved {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            CommandInteractionResolved::String(s) => serializer.serialize_str(&s),
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

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_false(v: &bool) -> bool {
    !v
}
//For the next update to Attachment
use std::any::Any;
use tokio::time::Instant as TokioInstant;

// pub fn is_none<T: Any>(v: &Option<T>) -> bool {
//     match v {
//         Some(_) => false,
//         None => true,
//     }
// }

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    // #[serde(skip_serializing_if = "is_false")]
    pub ephemeral: bool,
    // Added to allow for authentication params in new update
    ///// Timestamp indicating when the attachment URL will expire
    //#[serde(
    //    //default = "default_attachment_ex",
    //    skip_serializing_if = "is_none",
    //    with = "approx_instant"
    //)]
    //pub ex: Option<TokioInstant>,
    ///// Timestamp indicating when the URL was issued
    //#[serde(
    //    //default = "default_attachment_is",
    //    skip_serializing_if = "is_none",
    //    with = "approx_instant"
    //)]
    //pub is: Option<TokioInstant>,
    ///// Unique sinature that remains valid until [`ex`]: Self::Attachment::ex
    //#[serde(skip_serializing_if = "is_none")]
    //pub hm: Option<String>,
}

impl Default for Attachment {
    fn default() -> Attachment {
        Attachment {
            id: Default::default(),
            filename: Default::default(),
            height: Default::default(),
            proxy_url: Default::default(),
            size: Default::default(),
            url: Default::default(),
            width: Default::default(),
            content_type: Default::default(),
            ephemeral: Default::default(),
            // ex: default_attachment_ex(),
            // is: default_attachment_is(),
            // hm: default_attachment_hm(),
        }
    }
}

//use lazy_static::lazy_static;
////Until this exists hard-code these
//lazy_static! {
//    static ref DEFAULT_IS: TokioInstant = TokioInstant::now();
//    static ref DEFAULT_EX: TokioInstant = *DEFAULT_IS - std::time::Duration::new(86400_u64, 0_u32);
//    static ref DEFAULT_HM: String = String::from("bogus");
//}

impl From<SerenityAttachment> for Attachment {
    fn from(value: SerenityAttachment) -> Self {
        //let bogus_is = Instant::now();
        // 86400 seconds is a day, so a fair guess for now
        // Might be supported too by observed links
        //let bogus_ex = Instant::now() - std::time::Duration::new(86400_u64, 0_u32);
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
            //Until this exists hard-code these
            // is: value.is,
            // ex: value.ex,
            // hm: value.hm,
            // is: default_attachment_is(),
            // ex: default_attachment_ex(),
            // hm: default_attachment_hm(),
        }
    }
}

//Until this exists hard-code these
//fn default_attachment_is() -> Option<TokioInstant> {
//    Some(*DEFAULT_IS)
//}

////Until this exists hard-code these
//fn default_attachment_ex() -> Option<TokioInstant> {
//    Some(*DEFAULT_EX)
//}

////Until this exists hard-code these
//fn default_attachment_hm() -> Option<String> {
//    Some(DEFAULT_HM.to_string())
//}

//mod approx_instant {
//    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
//    use std::time::SystemTime;
//    use tokio::time::Instant;

//    pub fn serialize<S>(instant: &Option<Instant>, serializer: S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer,
//    {
//        let system_now = SystemTime::now();
//        let instant_now = Instant::now();
//        //N.B. This is only called if `instant` has some value
//        let approx = system_now - (instant_now - instant.unwrap());
//        approx.serialize(serializer)
//    }

//    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Instant>, D::Error>
//    where
//        D: Deserializer<'de>,
//    {
//        let de = SystemTime::deserialize(deserializer)?;
//        let system_now = SystemTime::now();
//        let instant_now = Instant::now();
//        let duration = system_now.duration_since(de).map_err(Error::custom)?;
//        let approx = instant_now - duration;
//        Ok(Some(approx))
//    }
//}

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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct ThreadMetadata {
    pub archived: bool,
    pub auto_archive_duration: AutoArchiveDuration,
    pub archive_timestamp: Option<Timestamp>,
    #[serde(default)]
    pub locked: bool,
    pub create_timestamp: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub invitable: bool,
}

impl From<Option<SerenityThreadMetadata>> for ThreadMetadata {
    fn from(upstream: Option<SerenityThreadMetadata>) -> Self {
        match upstream {
            Some(value) => Self {
                archived: value.archived,
                auto_archive_duration: value.auto_archive_duration,
                archive_timestamp: value.archive_timestamp,
                locked: value.locked,
                create_timestamp: value.create_timestamp,
                invitable: value.invitable,
            },
            None => Self::default(),
        }
    }
}

impl Default for ThreadMetadata {
    fn default() -> Self {
        Self {
            archived: false,
            auto_archive_duration: AutoArchiveDuration::None,
            archive_timestamp: None,
            locked: false,
            create_timestamp: None,
            invitable: true,
        }
    }
}

impl From<SerenityThreadMetadata> for ThreadMetadata {
    fn from(value: SerenityThreadMetadata) -> Self {
        Self {
            archived: value.archived,
            auto_archive_duration: value.auto_archive_duration,
            archive_timestamp: value.archive_timestamp,
            locked: value.locked,
            create_timestamp: value.create_timestamp,
            invitable: value.invitable,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct PartialChannel {
    pub id: ChannelId,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub kind: ChannelType,
    pub permissions: Option<Permissions>,
    pub thread_metadata: Option<ThreadMetadata>,
    pub parent_id: Option<ChannelId>,
}

impl Default for PartialChannel {
    fn default() -> Self {
        Self {
            id: serenity::model::id::ChannelId::default(),
            name: Some(String::default()),
            kind: ChannelType::Unknown(!0u8),
            permissions: Some(Permissions::default()),
            thread_metadata: serde_json::from_str(
                &serde_json::to_string(&ThreadMetadata::default()).expect(""),
            )
            .expect(""),
            parent_id: None,
        }
    }
}

impl From<SerenityPartialChannel> for PartialChannel {
    fn from(value: SerenityPartialChannel) -> Self {
        let thread_metadata = match value.thread_metadata {
            Some(tmd) => tmd,
            None => {
                serde_json::from_str(&serde_json::to_string(&ThreadMetadata::default()).expect(""))
                    .expect("")
            },
        };
        Self {
            id: value.id,
            name: value.name,
            kind: value.kind,
            permissions: value.permissions,
            thread_metadata: ThreadMetadata::from(thread_metadata).into(),
            parent_id: value.parent_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{
        prelude::prelude::{from_str, to_string},
        TestUser,
    };
    use serenity::all::User;
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
        let user_str = &to_string(&TestUser::default()).unwrap();
        let user = from_str::<User>(&user_str).unwrap();
        let attach_str = &to_string(&Attachment::default()).unwrap();
        let attach = from_str::<SerenityAttachment>(&attach_str).unwrap();
        let chan_str = to_string(&PartialChannel::default()).unwrap();
        let chan = from_str::<SerenityPartialChannel>(&chan_str).unwrap();
        let role_str = to_string(&Role::default()).unwrap();
        let role = from_str::<SerenityRole>(&role_str).unwrap();
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
        let test_interaction: CommandInteraction = CommandInteraction {
            id: InteractionId::new(!0u64),
            application_id: ApplicationId::new(!0u64),
            data: serde_json::from_str(
                &serde_json::to_string(&LocalCommandData::default()).expect(""),
            )
            .expect(""),
            guild_id: None,
            channel: None,
            channel_id: ChannelId::new(!0_u64),
            member: None,
            user: serde_json::from_str(&serde_json::to_string(&TestUser::default()).expect(""))
                .expect(""),
            token: String::from(""),
            version: 1u8,
            app_permissions: None,
            locale: String::from(""),
            guild_locale: None,
        };
        let _ = test_interaction.clone(); //derive(Clone)
        let ti_string = serde_json::to_string(&test_interaction); //derive(Serialize)
        let _ = serde_json::from_str::<CommandInteraction>(&ti_string.unwrap()); //impl Deserialize
        let _ = format!("{:?}", test_interaction); //derive(Debug)
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
