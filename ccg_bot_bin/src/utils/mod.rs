#[cfg(any(feature = "discord", feature = "full"))]
use serde::{Deserialize, Serialize};
#[cfg(any(feature = "discord", feature = "full"))]
use serenity::all::{
    GuildId, ImageHash, PartialMember, Permissions, PremiumType, RoleId, Timestamp, User, UserId,
    UserPublicFlags,
};
#[cfg(any(feature = "discord", feature = "full"))]
use std::num::NonZeroU16;

#[cfg(any(feature = "discord", feature = "full"))]
pub mod commandinteraction;
pub mod json;
pub mod prelude {
    #[cfg(any(feature = "discord", feature = "full"))]
    pub use super::commandinteraction;
    pub use super::json::*;
}

#[cfg(any(feature = "discord", feature = "full"))]
pub(crate) fn default_discriminator() -> Option<NonZeroU16> {
    NonZeroU16::new(1u16)
}

#[cfg(any(feature = "discord", feature = "full"))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TestUser {
    pub id: UserId,
    #[serde(rename = "username")]
    pub name: String,
    pub discriminator: Option<NonZeroU16>,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: bool,
    #[serde(default)]
    pub system: bool,
    #[serde(default)]
    pub mfa_enabled: bool,
    pub banner: Option<ImageHash>,
    pub locale: Option<String>,
    // We aren't requiring nor requesting this information and thereby these two field are None
    /// Whether the email on this account has been verified
    ///
    /// Requires `Scope::Email`
    #[serde(default)]
    pub verified: Option<bool>,
    // We aren't requiring nor requesting this information and thereby these two field are None
    /// The user's email
    ///
    /// Requires `Scope::Email`
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub flags: UserPublicFlags,
    pub premium_type: PremiumType,
    #[serde(default)]
    pub public_flags: Option<UserPublicFlags>,
    pub member: Option<Box<PartialMember>>,
}

#[cfg(any(feature = "discord", feature = "full"))]
impl Default for TestUser {
    fn default() -> Self {
        Self {
            id: UserId::new(1u64),
            name: String::from("TestUser"),
            discriminator: NonZeroU16::new(1u16),
            global_name: None,
            avatar: None,
            bot: true,
            system: false,
            mfa_enabled: false,
            banner: None,
            locale: Some(String::from("en-US")),
            verified: None,
            email: None,
            flags: UserPublicFlags::default(),
            premium_type: PremiumType::None,
            public_flags: None,
            member: None,
        }
    }
}

#[cfg(any(feature = "discord", feature = "full"))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LocalPartialMember {
    #[serde(default)]
    pub deaf: bool,
    pub joined_at: Option<Timestamp>,
    #[serde(default)]
    pub mute: bool,
    pub nick: Option<String>,
    pub roles: Vec<RoleId>,
    #[serde(default)]
    pub pending: bool,
    pub premium_since: Option<Timestamp>,
    pub guild_id: Option<GuildId>,
    pub user: Option<User>,
    pub permissions: Option<Permissions>,
}

#[cfg(any(feature = "discord", feature = "full"))]
fn default_pm() -> Option<Box<PartialMember>> {
    let local_partial_member = LocalPartialMember::default();
    let pm = serde_json::from_str(&serde_json::to_string(&local_partial_member).expect(""))
        .expect("Unable to deserialize LocalPartialMember into a serenity::all::PartialMember");
    Some(Box::new(pm))
}
