macro_rules! enum_number {
    ($name:ident { $($(#[$attr:meta])? $variant:ident $(,)? )* }) => {
        impl $name {
            #[inline]
            #[allow(dead_code)]
            pub fn num(&self) -> u64 {
                *self as u64
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
                where S: serde::Serializer
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where D: serde::Deserializer<'de>
            {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>)
                        -> std::fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> std::result::Result<$name, E>
                        where E: serde::de::Error
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( $(#[$attr])? v if v == $name::$variant as u64 => Ok($name::$variant), )*
                            _ => {
                                tracing::warn!("Unknown {} value: {}", stringify!($name), value);

                                Ok($name::Unknown)
                            }
                        }
                    }
                }

                // Deserialize the enum from a u64.
                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}
