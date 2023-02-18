//Crate imports
use crate::utils::json::{json, Value};

//Serenity imports
use serenity::builder::CreateEmbed;
use serenity::builder::CreateEmbedAuthor;
//use serenity::builder::CreateEmbedFooter;
//use serenity::utils::Color;

#[derive(Clone, Debug, Default)]
pub struct DiscordEmbedAuthor(pub CreateEmbedAuthor);

#[derive(Clone, Debug, Default)]
pub struct DiscordEmbed(pub CreateEmbed);

// Here's a two handy ways to figure out which functions to use to build our response type
// https://autocode.com/tools/discord/embed-builder
// https://cog-creators.github.io/discord-embed-sandbox
impl DiscordEmbed {
    pub fn new() -> DiscordEmbed {
        DiscordEmbed::default()
    }

    pub fn build(&mut self) -> CreateEmbed {
        self.clone().0
    }

    #[inline]
    pub fn color<C: Into<serenity::utils::Colour>>(&mut self, color: C) -> &mut Self {
        self._color(color.into())
    }

    fn _color(&mut self, color: serenity::utils::Colour) -> &mut Self {
        self.0 .0.insert("color", crate::utils::json::from_number(u64::from(color.0)));
        self
    }

    #[inline]
    pub fn field<T, U>(&mut self, name: T, value: U, inline: bool) -> &mut Self
    where
        T: ToString,
        U: ToString,
    {
        self._field(name.to_string(), value.to_string(), inline);
        self
    }

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

    fn url_object(&mut self, name: &'static str, url: String) -> &mut Self {
        let obj = json!({
            "url": url,
        });

        self.0 .0.insert(name, obj);
        self
    }

    #[inline]
    pub fn thumbnail<S: ToString>(&mut self, url: S) -> &mut Self {
        self.url_object("thumbnail", url.to_string());
        self
    }

    /*#[inline]
    pub fn title<D: ToString>(&mut self, title: D) -> &mut Self {
        self.0 .0.insert("title", Value::from(title.to_string()));
        self
    }*/
}
