#[cfg(all(any(feature = "discord", feature = "full"), test))]
pub type JsonMap = serde_json::Map<String, Value>;
pub type Value = serde_json::Value;
pub use serde_json::json;
pub use serde_json::Error as JsonError;
#[cfg(all(any(feature = "discord", feature = "full"), test))]
use std::collections::HashMap;
#[cfg(all(any(feature = "discord", feature = "full"), test))]
use std::hash::{BuildHasher, Hash};

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

pub mod prelude {
    pub use super::*;
    pub use serde_json::{
        from_reader, from_slice, from_str, from_value, to_string, to_string_pretty, to_value,
        to_vec, to_vec_pretty,
    };
}
