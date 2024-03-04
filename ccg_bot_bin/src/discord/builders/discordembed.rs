//!A builder that uses methods and structs from [serenity::builder](https://docs.rs/serenity/*/serenity/builder/index.html)
//!to create an Embed.
//!
//!This is usually used in a response to a [discord::commands](super::super::commands) being invoked.

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

    /// Set the author of the embed.
    pub fn author(self, author: serenity::all::CreateEmbedAuthor) -> Self {
        Self(self.0.author(author))
    }

    ///Sets the color to appear on the left side of the embed.
    #[inline]
    pub fn color<C: Into<serenity::all::Colour>>(self, color: C) -> Self {
        self._color(color.into())
    }

    #[doc(hidden)]
    fn _color(self, color: serenity::all::Colour) -> Self {
        Self(self.0.color(color))
    }

    ///Takes a name and value that impl ToString and a boolean as to whether to inline this field in the Embed.
    #[inline]
    pub fn field(self, name: impl Into<String>, value: impl Into<String>, inline: bool) -> Self {
        Self(self.0.field(name, value, inline))
    }

    #[inline]
    fn fields<N, V>(self, fields: impl IntoIterator<Item = (N, V, bool)>) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        {
            Self(self.0.fields(fields))
        }
    }

    #[doc(hidden)]
    fn url_object(self, url: impl Into<String>) -> Self {
        Self(self.0.url(url.into()))
    }

    /// Set the thumbnail of the embed. This only supports HTTP(S).
    #[inline]
    pub fn thumbnail(self, url: impl Into<String>) -> Self {
        Self(self.0.thumbnail(url))
    }

    /// Set the title of the embed.
    #[inline]
    pub fn title(self, title: impl Into<String>) -> Self {
        Self(self.0.title(title))
    }

    #[doc(hidden)]
    pub(crate) fn not_implimented() -> CreateEmbed {
        Self::new().title("Not Implimented").build()
    }
}
