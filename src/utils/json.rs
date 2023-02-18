pub type Value = serde_json::Value;
pub use serde_json::json;
pub use serde_json::Error as JsonError;

pub trait ToNumber {
    fn to_number(self) -> Value;
}

impl<T: Into<serde_json::Number>> ToNumber for T {
    fn to_number(self) -> Value {
        Value::Number(self.into())
    }
}

pub(crate) fn from_number(n: impl ToNumber) -> Value {
    n.to_number()
}

pub mod prelude {
    pub use super::*;
    pub use serde_json::{
        from_reader, from_slice, from_str, from_value, to_string, to_string_pretty, to_value,
        to_vec, to_vec_pretty,
    };
}
