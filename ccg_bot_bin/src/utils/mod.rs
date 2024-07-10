use serde::{Deserialize, Serialize};
use serenity::all::{
    GuildId, ImageHash, PartialMember, Permissions, PremiumType, RoleId, Timestamp, User, UserId,
    UserPublicFlags,
};

#[cfg(test)]
use std::any::type_name;
use std::num::NonZeroU16;

pub mod commandinteraction;
pub mod json;
#[cfg(test)]
pub use json::*;

#[allow(unused)]
pub(crate) fn default_discriminator() -> Option<NonZeroU16> {
    NonZeroU16::new(1u16)
}

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

#[allow(unused)]
fn default_pm() -> Option<Box<PartialMember>> {
    let local_partial_member = LocalPartialMember::default();
    let pm = serde_json::from_str(&serde_json::to_string(&local_partial_member).expect(""))
        .expect("Unable to deserialize LocalPartialMember into a serenity::all::PartialMember");
    Some(Box::new(pm))
}

pub mod approx_instant {
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Instant, SystemTime};

    #[allow(unused)]
    pub fn serialize<S>(instant: Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let system_now = SystemTime::now();
        let instant_now = Instant::now();
        // N.B. `instant` must(and can only) be in our past.
        let approx = system_now - (instant_now - instant);
        approx.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let de = SystemTime::deserialize(deserializer)?;
        let system_now = SystemTime::now();
        let instant_now = Instant::now();
        let duration = system_now.duration_since(de).map_err(Error::custom)?;
        let approx = instant_now - duration;
        Ok(approx)
    }
}

#[cfg(test)]
#[allow(unused)]
pub fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

#[doc(hidden)]
#[allow(unused)]
pub(crate) fn non_op_dbg(message: String) -> bool {
    use crate::debug;
    debug!(message);
    true
}

#[doc(hidden)]
#[allow(unused)]
pub(crate) fn non_op_trace(message: String) -> bool {
    use crate::trace;
    trace!(message);
    true
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::json::{from_str, to_string};

    #[test]
    fn discriminator_default() {
        let _ = default_discriminator();
    }

    #[test]
    fn derive_macros_testuser() {
        // Clone, Debug, Deserialize, Serialize, impl Default
        let testuser = TestUser::default();
        let testuser_cloned = testuser.clone();
        let testuser_string = to_string(&testuser).unwrap();
        let _: TestUser = from_str(&testuser_string).unwrap();
        dbg!(testuser_cloned);
    }

    #[test]
    fn derive_macros_localpartialmember() {
        // Clone, Debug, Default, Deserialize, Serialize
        let local_partial_member = LocalPartialMember::default();
        dbg!(&local_partial_member);
        let _ = local_partial_member.clone();
        let local_partial_member_string = to_string(&local_partial_member).unwrap();
        let _: LocalPartialMember = from_str(&local_partial_member_string).unwrap();
    }

    #[test]
    fn pm_default() {
        let _ = default_pm();
    }

    #[test]
    fn non_op_dbg_type_of() {
        let type_t = type_of(String::from("Test"));
        assert!(non_op_dbg(type_t.to_string()));
    }
}
