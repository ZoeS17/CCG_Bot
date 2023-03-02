//!A builder that uses methods and structs from [serenity::builder](https://docs.rs/serenity/*/serenity/builder/index.html)
//!to create an Embed.
//!
//!This is usually used in a response to a [discord::commands](super::super::commands) being invoked.

//Crate imports
use crate::utils::json::{json, Value};

//Serenity imports
use serenity::builder::CreateEmbed;

///Tuple struct to contain [`CreateEmbed`]
#[derive(Clone, Debug, Default)]
pub struct DiscordEmbed(pub CreateEmbed);

// Here's a two handy ways to figure out which functions to use to build our response type
// https://autocode.com/tools/discord/embed-builder
// https://cog-creators.github.io/discord-embed-sandbox
impl DiscordEmbed {
    ///Constructs a default [`DiscordEmbed`]
    pub fn new() -> DiscordEmbed {
        DiscordEmbed::default()
    }

    ///Consumes self to return a [`CreateEmbed`]
    pub fn build(&mut self) -> CreateEmbed {
        self.clone().0
    }

    ///Sets the color to appear on the left side of the embed.
    #[inline]
    pub fn color<C: Into<serenity::utils::Colour>>(&mut self, color: C) -> &mut Self {
        self._color(color.into())
    }

    #[doc(hidden)]
    fn _color(&mut self, color: serenity::utils::Colour) -> &mut Self {
        self.0 .0.insert("color", crate::utils::json::from_number(u64::from(color.0)));
        self
    }

    ///Takes a name and value that impl ToString and a boolean as to whether to inline this field in the Embed.
    #[inline]
    pub fn field<T, U>(&mut self, name: T, value: U, inline: bool) -> &mut Self
    where
        T: ToString,
        U: ToString,
    {
        self._field(name.to_string(), value.to_string(), inline);
        self
    }

    #[doc(hidden)]
    fn _field(&mut self, name: String, value: String, inline: bool) {
        {
            let entry =
                self.0 .0.entry("fields").or_insert_with(|| Value::from(Vec::<Value>::new()));

            if let Value::Array(ref mut inner) = *entry {
                inner.push(json!({
                    "inline": inline,
                    "name": name,
                    "value": value,
                }));
            }
        }
    }

    #[doc(hidden)]
    fn url_object(&mut self, name: &'static str, url: String) -> &mut Self {
        let obj = json!({
            "url": url,
        });

        self.0 .0.insert(name, obj);
        self
    }

    /// Set the thumbnail of the embed. This only supports HTTP(S).
    #[inline]
    pub fn thumbnail<S: ToString>(&mut self, url: S) -> &mut Self {
        self.url_object("thumbnail", url.to_string());
        self
    }

    /// Set the title of the embed.
    #[inline]
    pub fn title<D: ToString>(&mut self, title: D) -> &mut Self {
        self.0 .0.insert("title", Value::from(title.to_string()));
        self
    }
}
