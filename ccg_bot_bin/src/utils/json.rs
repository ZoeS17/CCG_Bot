#[cfg(all(any(feature = "discord", feature = "full"), test))]
pub type JsonMap = serde_json::Map<String, Value>;
pub type Value = serde_json::Value;
pub use serde_json::json;
pub use serde_json::Error as JsonError;
#[cfg(all(any(feature = "discord", feature = "full"), test))]
use serenity::all::Embed;
#[cfg(all(any(feature = "discord", feature = "full"), test))]
pub use serenity::builder::CreateEmbed;
#[cfg(all(any(feature = "discord", feature = "full"), test))]
use std::collections::HashMap;
#[cfg(all(any(feature = "discord", feature = "full"), test))]
use std::hash::{BuildHasher, Hash};

#[cfg(all(any(feature = "discord", feature = "full"), test))]
use serde_path_to_error::deserialize as serde_path_to_error_deserialize;

pub trait ToNumber {
    fn to_number(self) -> Value;
}

impl<T: Into<serde_json::Number>> ToNumber for T {
    fn to_number(self) -> Value {
        Value::Number(self.into())
    }
}

#[cfg(any(feature = "discord", feature = "full"))]
pub fn from_number(n: impl ToNumber) -> Value {
    n.to_number()
}

#[cfg(all(any(feature = "discord", feature = "full"), test))]
pub fn hashmap_to_json_map<H, T>(map: HashMap<T, Value, H>) -> JsonMap
where
    H: BuildHasher,
    T: Eq + Hash + ToString,
{
    map.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

#[cfg(all(any(feature = "discord", feature = "full"), test))]
pub fn createembed_to_embed(c_embed: CreateEmbed) -> Embed {
    let c_embed_string = serde_json::to_string(&c_embed).expect("");
    serde_json::from_str::<Embed>(&c_embed_string).expect("")
}

#[cfg(all(any(feature = "discord", feature = "full"), test))]
pub fn createembed_to_json_map(c_embed: CreateEmbed) -> JsonMap {
    let c_embed_string = serde_json::to_string(&c_embed).expect("");
    let embed = serde_json::from_str::<Embed>(&c_embed_string).expect("");
    let map =
        parse_json::<JsonMap>(&serde_json::to_string(&embed).expect("").split_off(0)).expect("");
    map
}

/// A deserialization error
#[derive(Debug)]
#[non_exhaustive]
pub enum DeserError {
    PathError {
        /// Path to where the erroring key/value is
        path: String,
        /// Error for the key/value
        error: serde_json::Error,
    },
}

#[cfg(all(any(feature = "discord", feature = "full"), test))]
fn parse_json<'a, T: serde::Deserialize<'a>>(s: &'a str) -> Result<T, DeserError> {
    let jd = &mut serde_json::Deserializer::from_str(s);
    serde_path_to_error_deserialize(jd)
        .map_err(|e| DeserError::PathError { path: e.path().to_string(), error: e.into_inner() })
}

pub trait AnyExt {
    fn type_name(&self) -> &'static str;
}

impl<T> AnyExt for T {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

pub mod prelude {
    pub use super::*;
    pub use serde_json::{
        from_reader, from_slice, from_str, from_value, to_string, to_string_pretty, to_value,
        to_vec, to_vec_pretty,
    };
}
