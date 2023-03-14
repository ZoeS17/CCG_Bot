#![deny(clippy::dbg_macro)]
#![deny(clippy::missing_safety_doc)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
//Crate doc
#![doc = include_str!("../../README.md")]

#[macro_use]
extern crate tracing;

//crate
//use ccg_bot_sys;
use config::Config;

// serde
use serde_json::Error as JsonError;

//std
use std::env;
use std::error::Error as StdError;
use std::fmt::{self, Error as FormatError};
use std::io::Error as IoError;
use std::result::Result as StdResult;

#[cfg(test)]
mod tests;

mod config;
#[cfg(any(feature = "discord", feature = "full"))]
mod discord;
#[macro_use]
mod internals;

#[cfg(any(feature = "twitch", feature = "full"))]
mod twitch;
mod utils;

#[cfg(any(feature = "discord", feature = "full"))]
use discord::DiscordErr as DiscordError;
#[cfg(any(feature = "twitch", feature = "full"))]
use twitch::TwitchErr as TwitchError;

///This is an enum of all the error types this crate handles
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    ///This is a tuple struct of [`FormatError`]
    Format(FormatError),
    ///This is a tuple struct of [`IoError`]
    Io(IoError),
    ///This is a tuple struct of [`JsonError`]
    Json(JsonError),
    ///This is a tuple struct of [`DiscordError`] and is behind the `discord` feature flag which is enabled by default
    #[cfg(any(feature = "discord", feature = "full"))]
    Discord(DiscordError),
    ///This is a tuple struct of [`TwitchError`] and is behind the `twitch` feature flag which is disabled by default
    #[cfg(any(feature = "twitch", feature = "full"))]
    Twitch(TwitchError),
}

impl From<FormatError> for Error {
    fn from(e: FormatError) -> Self {
        Error::Format(e)
    }
}
impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}
impl From<JsonError> for Error {
    fn from(e: JsonError) -> Self {
        Error::Json(e)
    }
}
#[cfg(any(feature = "discord", feature = "full"))]
impl From<DiscordError> for Error {
    fn from(e: DiscordError) -> Self {
        Error::Discord(e)
    }
}

#[cfg(any(feature = "twitch", feature = "full"))]
impl From<TwitchError> for Error {
    fn from(e: TwitchError) -> Self {
        Error::Twitch(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Format(inner) => fmt::Display::fmt(&inner, f),
            Self::Io(inner) => fmt::Display::fmt(&inner, f),
            Self::Json(inner) => fmt::Display::fmt(&inner, f),
            #[cfg(any(feature = "discord", feature = "full"))]
            Self::Discord(inner) => fmt::Display::fmt(&inner, f),
            #[cfg(any(feature = "twitch", feature = "full"))]
            Self::Twitch(inner) => fmt::Display::fmt(&inner, f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Format(inner) => Some(inner),
            Self::Io(inner) => Some(inner),
            Self::Json(inner) => Some(inner),
            #[cfg(any(feature = "discord", feature = "full"))]
            Self::Discord(inner) => Some(inner),
            #[cfg(any(feature = "twitch", feature = "full"))]
            Self::Twitch(inner) => Some(inner),
        }
    }
}

#[tokio::main]
async fn main() -> StdResult<(), Box<dyn StdError + Send + Sync>> {
    let mut log_var = String::from("");
    for (k, v) in env::vars() {
        if k == "RUST_LOG" {
            log_var = v.to_string();
        }
    }
    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`, but for production, use the variable defined below.
    if !log_var.is_empty() {
        env::set_var("RUST_LOG", format!("warn,ccg_bot={},meio=error", log_var));
    } else {
        env::set_var("RUST_LOG", "warn,CCG_Bot=warn,meio=error");
    }
    tracing_subscriber::fmt::init();
    #[cfg(any(feature = "discord", feature = "twitch", feature = "full"))]
    let config: Config = Config::new();
    #[cfg(any(feature = "discord", feature = "full"))]
    let discord_handle = tokio::spawn(discord::new(config.clone()));
    #[cfg(any(feature = "twitch", feature = "full"))]
    let twitch_handle = tokio::spawn(twitch::new(config));
    #[cfg(any(feature = "full", all(feature = "twitch", feature = "discord")))]
    let (_first, _second) = tokio::join!(discord_handle, twitch_handle);
    Ok(())
}
