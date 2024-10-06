//!This way be Twitch logging

//crate
use crate::config::Config;
//skip reordering to allow easy reference to verbosity(from least to most)
#[rustfmt::skip]
use crate::{info, debug, trace};
#[cfg(not(test))]
use crate::error;
#[cfg(not(test))]
use crate::utils::non_op_trace;

//eyre
#[cfg(not(test))]
use eyre::WrapErr;

//std
use std::fmt;

//twitch_api
#[cfg(not(test))]
use twitch_api::{client::ClientDefault, HelixClient};

//twitch_irc
use twitch_irc::login::RefreshingLoginCredentials;
#[cfg(not(test))]
use twitch_irc::message::{IRCMessage, JoinMessage, PrivmsgMessage, ServerMessage};
use twitch_irc::{SecureTCPTransport, TwitchIRCClient};

//module(s)
pub(crate) mod api;
mod commands;
// #[cfg(not(test))]
pub(crate) mod eventsub;
#[doc(hidden)]
pub(crate) mod tokens;

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
            TwitchErr::Other(ref err) => write!(f, "Other error: {err}"),
        }
    }
}

#[doc(hidden)]
impl std::error::Error for TwitchErr {}

#[doc(hidden)]
impl From<std::env::VarError> for TwitchErr {
    fn from(err: std::env::VarError) -> TwitchErr {
        TwitchErr::VarErr(err)
    }
}

#[doc(hidden)]
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
        "debug" => debug!("Received message: {:?}", message),
        "trace" => trace!("Received message: {:?}", message),
        _ => panic!(),
    }
}

///Creates a new chat listener for channels in your config.toml
pub async fn new(config: Config) -> eyre::Result<Handler> {
    // these credentials can be generated for your app at https://dev.twitch.tv/console/apps
    // the bot's username will be set based on your config
    let cfg = config.clone();
    let prefix = "TWITCH".to_string();

    #[cfg(not(test))]
    let mut storage =
        tokens::BotTokenStorage::init(&mut tokens::BotTokenStorage::default(), prefix);
    #[cfg(test)]
    let storage = tokens::BotTokenStorage::init(&mut tokens::BotTokenStorage::default(), prefix);
    let client_config = storage.clone().client_config(cfg.clone()).await;
    #[cfg(not(test))]
    let token = storage.token().await;
    #[cfg(not(test))]
    let app_token = tokens::AppToken::new().await;

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
    {
        let client_clone = client.clone();
        let mut join_handles = vec![];
        join_handles.push(tokio::spawn(async move {
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
                    ServerMessage::Join { .. } => {
                        let m = JoinMessage::try_from(Into::<IRCMessage>::into(message.clone()))
                            .unwrap();
                        println!("[twitch / #{}] {} joined", m.channel_login, m.user_login)
                    },
                    ServerMessage::Notice { .. } => {
                        parse_message("trace", format!("{:?}", message));
                    },
                    ServerMessage::Part { .. } => {
                        parse_message("info", format!("{:?}", message));
                    },
                    ServerMessage::Ping { .. } => {
                        parse_message("trace", format!("{:?}", message));
                    },
                    ServerMessage::Pong { .. } => {
                        parse_message("trace", format!("{:?}", message));
                    },
                    ServerMessage::Privmsg { .. } => {
                        let m = PrivmsgMessage::try_from(Into::<IRCMessage>::into(message.clone()))
                            .unwrap();
                        commands::parse_command(message, client_clone.clone()).await;
                        println!(
                            "[twitch / {}] {}: {}",
                            m.channel_login, m.sender.login, m.message_text
                        )
                    },
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
                        // Should this be left at debug or should it be trace because of reporting safety?
                        // We don't want users to accidentaly leak their whispers.
                        commands::parse_command(message, client_clone.clone()).await;
                    },
                    _ => eprintln!("received unexpected message variant {:?}", message),
                }
            }
        }));
        for channel in &cfg.twitch_channels {
            let un = token.clone().name.take();
            debug_assert!(non_op_trace(format!("`{}` ?= `{}`", un, channel.to_lowercase())));
            if channel.to_lowercase() == un {
                let reqwest_client = <reqwest::Client>::default_client_with_name(Some(
                    "twitch-rs/eventsub"
                        .parse()
                        .wrap_err_with(|| "when creating header name")
                        .unwrap(),
                ))
                .wrap_err_with(|| "when creating client")?;
                let helix_client: HelixClient<_> = twitch_api::HelixClient::with_client(
                    // twitch_api::client::ClientDefault::default_client_with_name(Some(
                    // <reqwest::Client>::default_client_with_name(Some(
                    //     "twitch-rs/eventsub"
                    //         .parse()
                    //         .wrap_err_with(|| "when creating header name")
                    //         .unwrap(),
                    // ))
                    // .wrap_err_with(|| "when creating client")?,
                    reqwest_client,
                );
                let id = helix_client
                    .get_user_from_login(channel, &token)
                    .await?
                    .ok_or_else(|| eyre::eyre!("no user found with name {channel}"))?
                    .id;
                let websocket_client = eventsub::WebsocketClient {
                    session_id: None,
                    user_token: token.clone(),
                    app_token: app_token.clone(),
                    client: helix_client,
                    user_id: id,
                    connect_url: twitch_api::TWITCH_EVENTSUB_WEBSOCKET_URL
                        .clone()
                        .as_str()
                        .parse()
                        .unwrap(),
                };

                //TODO: Figure out why this bails once, likely due to nested await's
                join_handles.push(tokio::spawn(async move {
                    match websocket_client.run().await {
                        Ok(_) => (),
                        Err(e) => error!("{e}"),
                    }
                }));
            }
            client.join(channel.to_owned().to_lowercase()).unwrap();
        }
        for handle in join_handles {
            handle.await.unwrap();
        }
    }
    Ok(Handler(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_message() {
        super::parse_message("info", "Test info");
        super::parse_message("debug", "Test debug");
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
            database_url: "".to_string(),
            discord_guildid: "".to_string(),
            discord_token: "".to_string(),
            twitch_bot_name: "".to_string(),
            twitch_channels: vec![],
            twitch_client_id: "".to_string(),
            twitch_client_secret: "".to_string(),
            twitch_redirect_url: "".to_string(),
            bot_admins: vec![],
        });
        let _ = format!("{:?}", handle);
    }

    #[test]
    fn fmt_twitch_err_other_error() {
        let other_error = super::TwitchErr::Other("Other Error".to_string());
        let _ = format!("{}", other_error);
    }

    #[test]
    fn fmt_twitch_err_failedtoparse_error() {
        let failed_to_parse = super::TwitchErr::FailedToParse {
            key: "TEST_FAIL".to_string(),
            value: "failed to parse".to_string(),
            error: Some("Failed successfully".to_string()),
        };
        let _ = format!("{}", failed_to_parse);
    }
}
