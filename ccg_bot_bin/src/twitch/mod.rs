//!This way be Twitch logging

//crate
use crate::config::Config;

//std
use std::error;
use std::fmt;

//twitch_irc
use twitch_irc::login::RefreshingLoginCredentials;
#[cfg(not(test))]
use twitch_irc::message::{IRCMessage, JoinMessage, PrivmsgMessage, ServerMessage};
use twitch_irc::{SecureTCPTransport, TwitchIRCClient};

//module(s)
#[doc(hidden)]
mod tokens;

#[derive(Debug)]
#[doc(hidden)]
pub struct Handler(pub Config);

#[non_exhaustive]
#[derive(Debug)]
#[doc(hidden)]
pub enum TwitchErr {
    FailedToParse { key: String, value: String, error: Option<String> },
    Other(String),
    VarErr(std::env::VarError),
}

#[doc(hidden)]
impl fmt::Display for TwitchErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TwitchErr::FailedToParse { key, value, error } => {
                let mut s = format!("Failed to parse {} as {}", value, key);
                if let Some(e) = error {
                    s.push_str(&format!(": {}", e));
                }
                write!(f, "{}", s)
            },
            TwitchErr::VarErr(ref err) => write!(f, "Var error: {err}"),
            TwitchErr::Other(ref err) => write!(f, "Var error: {err}"),
        }
    }
}

#[doc(hidden)]
impl error::Error for TwitchErr {
    // fn cause(&self) -> Option<&dyn error::Error> {
    //     match *self {
    //         TwitchErr::VarErr(ref err) => Some(err),
    //     }
    // }
}

#[doc(hidden)]
impl From<std::env::VarError> for TwitchErr {
    fn from(err: std::env::VarError) -> TwitchErr {
        TwitchErr::VarErr(err)
    }
}

fn get_stacktrace(e: &dyn std::error::Error) -> String {
    let mut s = vec![];
    let mut source = Some(e);
    while let Some(e) = source {
        s.push(e.to_string());
        source = e.source();
    }
    s.join("\n")
}

#[doc(hidden)]
impl From<&dyn std::error::Error> for TwitchErr {
    fn from(err: &dyn std::error::Error) -> Self {
        TwitchErr::Other(get_stacktrace(err))
    }
}

#[doc(hidden)]
pub(crate) fn parse_message<L: AsRef<str> + std::fmt::Debug, M: AsRef<str> + std::fmt::Debug>(
    level: L,
    message: M,
) {
    match level.as_ref() {
        "info" => info!("Received message: {:?}", message),
        "trace" => trace!("Received message: {:?}", message),
        _ => panic!(),
    }
}

///Creates a new chat listener for channels in your config.toml
pub async fn new(config: Config) -> Result<Handler, std::env::VarError> {
    // these credentials can be generated for your app at https://dev.twitch.tv/console/apps
    // the bot's username will be fetched based on your access token
    let cfg = config.clone();
    let prefix = Some("TWITCH".to_string());

    let storage = tokens::BotTokenStorage::new(&mut tokens::BotTokenStorage::default(), prefix);
    let client_config = storage.client_config(cfg.clone());

    #[cfg(not(test))]
    let (mut incoming_messages, client) = TwitchIRCClient::<
        SecureTCPTransport,
        RefreshingLoginCredentials<tokens::BotTokenStorage>,
    >::new(client_config);
    #[cfg(test)]
    let (_incoming_messages, _client) = TwitchIRCClient::<
        SecureTCPTransport,
        RefreshingLoginCredentials<tokens::BotTokenStorage>,
    >::new(client_config);
    #[cfg(not(test))]
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                //Match each of the non-exhaustive cases explictly so we can error on unknown ones
                ServerMessage::ClearChat { .. } => {
                    parse_message("trace", format!("{:?}", message));
                },
                ServerMessage::ClearMsg { .. } => {
                    parse_message("trace", format!("{:?}", message));
                },
                ServerMessage::Generic { .. } => {
                    parse_message("trace", format!("{:?}", message));
                },
                ServerMessage::GlobalUserState { .. } => {
                    parse_message("trace", format!("{:?}", message));
                },
                ServerMessage::Join { .. } => println!(
                    "[twitch / #{}] {} joined",
                    JoinMessage::try_from(Into::<IRCMessage>::into(message.clone()))
                        .unwrap()
                        .channel_login,
                    JoinMessage::try_from(Into::<IRCMessage>::into(message.clone()))
                        .unwrap()
                        .user_login,
                ),
                ServerMessage::Notice { .. } => {
                    parse_message("trace", format!("{:?}", message));
                },
                ServerMessage::Part { .. } => {
                    parse_message("info", format!("{:?}", message));
                }, //FIXME: parse part messages like we do privmsg's
                ServerMessage::Ping { .. } => {
                    parse_message("trace", format!("{:?}", message));
                },
                ServerMessage::Pong { .. } => {
                    parse_message("trace", format!("{:?}", message));
                },
                ServerMessage::Privmsg { .. } => println!(
                    "[twitch / {}] {}: {}",
                    PrivmsgMessage::try_from(Into::<IRCMessage>::into(message.clone()))
                        .unwrap()
                        .channel_login,
                    PrivmsgMessage::try_from(Into::<IRCMessage>::into(message.clone()))
                        .unwrap()
                        .sender
                        .login,
                    PrivmsgMessage::try_from(Into::<IRCMessage>::into(message))
                        .unwrap()
                        .message_text
                ),
                ServerMessage::Reconnect { .. } => {
                    parse_message("trace", format!("{:?}", message));
                },
                ServerMessage::RoomState { .. } => {
                    parse_message("trace", format!("{:?}", message));
                },
                ServerMessage::UserNotice { .. } => {
                    parse_message("trace", format!("{:?}", message));
                },
                ServerMessage::UserState { .. } => {
                    parse_message("trace", format!("{:?}", message));
                },
                ServerMessage::Whisper { .. } => {
                    parse_message("debug", format!("{:?}", message));
                },
                _ => eprintln!("received unexpected message variant {:?}", message),
            }
        }
    });
    #[cfg(not(test))]
    for channel in &cfg.twitch_channels {
        client.join(channel.to_owned().to_lowercase()).unwrap();
    }
    #[cfg(not(test))]
    join_handle.await.unwrap();
    Ok(Handler(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_message() {
        super::parse_message("info", "Test info");
        super::parse_message("trace", "Test trace");
    }

    #[test]
    #[should_panic]
    fn parse_message_bogus() {
        super::parse_message("foo", "bar");
    }

    #[test]
    fn fmt_twitch_err_var_error() {
        let var_error = std::env::VarError::NotPresent;
        let e: TwitchErr = Into::<TwitchErr>::into(var_error);
        let _ = format!("{:?}", &e);
        let _ = format!("{}", e);
    }

    #[test]
    fn debug_handler() {
        let handle = Handler(Config {
            #[cfg(any(feature = "discord", feature = "full"))]
            discord_guildid: "".to_string(),
            #[cfg(any(feature = "discord", feature = "full"))]
            discord_token: "".to_string(),
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch_bot_name: "".to_string(),
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch_channels: vec![],
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch_client_id: "".to_string(),
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch_client_secret: "".to_string(),
        });
        let _ = format!("{:?}", handle);
    }
}
