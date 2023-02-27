//std
use std::error;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

//twitch_irc
use twitch_irc::login::{LoginCredentials, RefreshingLoginCredentials};
use twitch_irc::message::{PrivmsgMessage, ServerMessage};
use twitch_irc::{ClientConfig, SecureWSTransport, TwitchIRCClient};

//nom
use nom::{branch::alt, bytes::complete::tag_no_case, Finish};

//serde
use serde_json;

//tracing
use tracing::{debug, error, info, instrument, trace, warn};

//misc
use governor::{Quota, RateLimiter};

//modules
mod tokens;

#[non_exhaustive]
#[derive(Debug)]
pub enum TwitchErr {
    VarErr(std::env::VarError),
}

impl fmt::Display for TwitchErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TwitchErr::VarErr(ref err) => write!(f, "Var error: {err}"),
        }
    }
}

impl error::Error for TwitchErr {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            TwitchErr::VarErr(ref err) => Some(err),
        }
    }
}

impl From<std::env::VarError> for TwitchErr {
    fn from(err: std::env::VarError) -> TwitchErr {
        TwitchErr::VarErr(err)
    }
}

#[tokio::main]
pub async fn main() {}
