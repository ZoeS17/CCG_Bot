#[cfg(test)]
#[allow(unused)]
pub type JsonMap = serde_json::Map<String, Value>;
pub type Value = serde_json::Value;
pub use serde_json::Error as JsonError;

#[cfg(test)]
use serenity::all::Embed;
#[cfg(test)]
pub use serenity::builder::CreateEmbed;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::hash::{BuildHasher, Hash};

#[cfg(test)]
use serde_path_to_error::deserialize as serde_path_to_error_deserialize;

pub trait ToNumber {
    fn to_number(self) -> Value;
}

impl<T: Into<serde_json::Number>> ToNumber for T {
    fn to_number(self) -> Value {
        Value::Number(self.into())
    }
}

pub fn from_number(n: impl ToNumber) -> Value {
    n.to_number()
}

#[cfg(test)]
#[allow(unused)]
pub fn hashmap_to_json_map<H, T>(map: HashMap<T, Value, H>) -> JsonMap
where
    H: BuildHasher,
    T: Eq + Hash + ToString,
{
    map.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

#[cfg(test)]
pub fn createembed_to_embed(c_embed: CreateEmbed) -> Embed {
    let c_embed_string = serde_json::to_string(&c_embed).expect("");
    serde_json::from_str::<Embed>(&c_embed_string).expect("")
}

#[cfg(test)]
pub fn createembed_to_json_map(c_embed: CreateEmbed) -> JsonMap {
    let c_embed_string = serde_json::to_string(&c_embed).expect("");
    let embed = serde_json::from_str::<Embed>(&c_embed_string).expect("");
    let map =
        parse_json::<JsonMap>(&serde_json::to_string(&embed).expect("").split_off(0)).expect("");
    map
}

/// A deserialization error
#[allow(unused)]
#[derive(Debug)]
#[non_exhaustive]
pub enum DeserError {
    PathError {
        /// Path to where the erroring key/value is
        path: String,
        /// Error for the key/value
        error: JsonError,
    },
}

#[cfg(test)]
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

#[cfg(test)]
pub use serde_json::{
    // from_reader, from_slice, from_str, from_value, to_string, to_string_pretty, to_value,
    // to_vec, to_vec_pretty,
    from_str,
    to_string,
};

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct TestNewtype(i32);

    #[test]
    fn to_type_name() {
        let new_type = 1_i32;
        dbg!(new_type.type_name());
    }

    #[test]
    fn from_createembed_to_embed() {
        let create_embed = CreateEmbed::new();
        let _ = createembed_to_embed(create_embed);
    }

    #[test]
    fn from_createembed_to_json_map() {
        let create_embed = CreateEmbed::new();
        let _ = createembed_to_json_map(create_embed);
    }

    #[test]
    fn i32_to_number() {
        let new_type = 2_i32;
        let _ = new_type.to_number();
    }

    #[test]
    fn i32_from_number() {
        let new_type = 2_i32;
        let _ = from_number(new_type);
    }

    #[test]
    fn newtype_deser_ialize() {
        let new_type = TestNewtype(1_i32);
        let new_type_str = to_string(&new_type).unwrap();
        let _: TestNewtype = from_str(&new_type_str).unwrap();
    }

    #[test]
    fn jsonmap_error() {
        let jsonmap_str = String::from("not a number");
        let _: DeserError = parse_json::<TestNewtype>(&jsonmap_str).unwrap_err();
    }

    #[test]
    fn hashmap_to_jsonmap() {
        let mut hashmap: HashMap<String, Value> = HashMap::new();
        hashmap.insert("Three".to_string(), 3_i32.to_number());
        let _ = hashmap_to_json_map(hashmap);
    }
}
