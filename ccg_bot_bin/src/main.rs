#![deny(clippy::dbg_macro)]
#![deny(clippy::missing_safety_doc)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
//Crate doc
#![doc = include_str!("../../README.md")]

#[cfg(any(feature = "discord", feature = "full", feature = "twitch"))]
#[macro_use]
extern crate tracing;

//crate
//use ccg_bot_sys;
#[cfg(any(feature = "discord", feature = "full", feature = "twitch"))]
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

#[cfg(not(any(feature = "discord", feature = "twitch", feature = "full", test)))]
fn main() {
    std::compile_error!("Please rebuild with --feature [discord | twitch | full ] ");
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
        env::set_var(
            "RUST_LOG",
            format!(
                "warn,ccg_bot={},meio=error,twitch_irc={},reqwest={}",
                &log_var, &log_var, log_var
            ),
        );
    } else {
        env::set_var("RUST_LOG", "warn,CCG_Bot=warn,meio=error,twitch_irc=warn");
    }
    tracing_subscriber::fmt::init();
    #[cfg(any(feature = "discord", feature = "twitch", feature = "full"))]
    let config: Config = Config::new();
    #[cfg(any(feature = "full", all(feature = "twitch", feature = "discord")))]
    let discord_handle = discord::new(config.clone());
    #[cfg(any(feature = "full", all(feature = "twitch", feature = "discord")))]
    let twitch_handle = twitch::new(config.clone());
    #[cfg(all(feature = "discord", not(any(feature = "full", feature = "twitch"))))]
    match discord::new(config.clone()).await {
        Ok(_) => (),
        Err(e) => eprintln!("{:?}", e),
    };
    #[cfg(all(feature = "twitch", not(any(feature = "full", feature = "discord"))))]
    match twitch::new(config.clone()).await {
        Ok(_) => (),
        Err(e) => eprintln!("{:?}", e),
    };
    #[cfg(any(feature = "full", all(feature = "twitch", feature = "discord")))]
    let (_first, _second) = tokio::join!(discord_handle, twitch_handle);
    Ok(())
}

#[cfg(test)]
mod main_tests {
    use super::*;
    use crate::utils::json::prelude::from_str;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct JsonErrorStruct {
        foo: String,
        bar: Vec<String>,
    }

    fn statisfy_clippy() {
        // why do I have to read this bogus struct for your goofy dead code analysis
        let jes = JsonErrorStruct { foo: "".to_string(), bar: vec!["".to_string()] };
        let _foo = format!("{:?}", jes.foo);
        let _bar = format!("{:?}", jes.bar);
    }

    #[test]
    fn derives_for_error() {
        let _ = statisfy_clippy();
        let e0 = Error::Format(FormatError);
        let _ = format!("{:?}", e0);
        let e1 = Error::Io(IoError::new(std::io::ErrorKind::Other, "test error"));
        let _ = format!("{:?}", e1);
        let je: Result<JsonErrorStruct, JsonError> = from_str(r#"""{"foo":"", "bar": ""}"""#);
        let e2 = Error::Json(je.unwrap_err());
        let _ = format!("{:?}", e2);
        #[cfg(any(feature = "discord", feature = "full"))]
        {
            let e3 = Error::Discord(DiscordError::VarErr(env::VarError::NotPresent));
            let _ = format!("{:?}", e3);
        }
        #[cfg(any(feature = "twitch", feature = "full"))]
        {
            let e4 = Error::Twitch(TwitchError::VarErr(env::VarError::NotPresent));
            let _ = format!("{:?}", e4);
        }
    }

    #[test]
    fn impl_from_formaterror_for_error() {
        let _ = Error::from(FormatError);
        let _: Error = FormatError.into();
    }

    #[test]
    fn impl_from_ioerror_for_error() {
        let e = IoError::new(std::io::ErrorKind::Other, "test error");
        let e2 = IoError::new(std::io::ErrorKind::Other, "test error");
        let _ = Error::from(e);
        let _: Error = e2.into();
    }

    #[test]
    fn impl_from_jsonerror_for_error() {
        let je: Result<JsonErrorStruct, JsonError> = from_str(r#"""{"foo":"", "bar": ""}"""#);
        let e = JsonError::from(je.unwrap_err());
        let je2: Result<JsonErrorStruct, JsonError> = from_str(r#"""{"foo":"", "bar": ""}"""#);
        let e2 = JsonError::from(je2.unwrap_err());
        let _ = Error::from(e);
        let _: Error = e2.into();
    }

    #[cfg(any(feature = "discord", feature = "full"))]
    #[test]
    fn impl_from_discorderror_for_error() {
        let e = DiscordError::VarErr(env::VarError::NotPresent);
        let e2 = DiscordError::VarErr(env::VarError::NotPresent);
        let _ = Error::from(e);
        let _: Error = e2.into();
    }

    #[cfg(any(feature = "twitch", feature = "full"))]
    #[test]
    fn impl_from_twitcherror_for_error() {
        let e = TwitchError::VarErr(env::VarError::NotPresent);
        let e2 = TwitchError::VarErr(env::VarError::NotPresent);
        let _ = Error::from(e);
        let _: Error = e2.into();
    }

    #[test]
    fn impl_display_for_error() {
        let e0 = Error::Format(FormatError);
        let _ = format!("{}", e0);
        let e1 = Error::Io(IoError::new(std::io::ErrorKind::Other, "test error"));
        let _ = format!("{}", e1);
        let je: Result<JsonErrorStruct, JsonError> = from_str(r#"""{"foo":"", "bar": ""}"""#);
        let e2 = Error::Json(je.unwrap_err());
        let _ = format!("{}", e2);
        #[cfg(any(feature = "discord", feature = "full"))]
        {
            let e3 = Error::Discord(DiscordError::VarErr(env::VarError::NotPresent));
            let _ = format!("{}", e3);
        }
        #[cfg(any(feature = "twitch", feature = "full"))]
        {
            let e4 = Error::Twitch(TwitchError::VarErr(env::VarError::NotPresent));
            let _ = format!("{}", e4);
        }
    }

    #[test]
    fn impl_stderror_for_error() {
        let _ = Error::Format(FormatError).source();
        let _ = Error::Io(IoError::new(std::io::ErrorKind::Other, "test error")).source();
        let rje: Result<JsonErrorStruct, JsonError> = from_str(r#"""{"foo":"", "bar": ""}"""#);
        let je = rje.unwrap_err();
        let _ = Error::Json(je).source();
        #[cfg(any(feature = "discord", feature = "full"))]
        let _ = Error::Discord(DiscordError::VarErr(env::VarError::NotPresent)).source();
    }
}
