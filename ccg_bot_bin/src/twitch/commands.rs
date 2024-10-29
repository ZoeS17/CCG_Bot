use twitch_irc::login::RefreshingLoginCredentials;
use twitch_irc::message::{Badge, IRCMessage, PrivmsgMessage, ServerMessage, WhisperMessage};
use twitch_irc::{transport::tcp::SecureTCPTransport, TwitchIRCClient};

use super::tokens::BotTokenStorage;

//command each in a module
mod link;
mod ping;

// use crate::debug;

pub fn has_mod_rights(message: PrivmsgMessage) -> bool {
    if message.badges.contains(&Badge { name: "moderator".to_string(), version: "1".to_string() })
        || message
            .badges
            .contains(&Badge { name: "broadcaster".to_string(), version: "1".to_string() })
    {
        return true;
    }
    false
}

pub fn has_bot_admin_rights(user_login: String, config: &crate::Config) -> bool {
    let bot_admins = &config.bot_admins;

    bot_admins.contains(&user_login)
}

pub async fn parse_command(
    message: ServerMessage,
    irc_client: TwitchIRCClient<
        SecureTCPTransport<twitch_irc::transport::tcp::TLS>,
        RefreshingLoginCredentials<BotTokenStorage>,
    >,
) {
    match message {
        // pseudo-default case
        ServerMessage::Privmsg { .. } => {
            let m = PrivmsgMessage::try_from(Into::<IRCMessage>::into(message.clone())).unwrap();
            #[allow(clippy::collapsible_if)]
            if has_mod_rights(m.to_owned())
                | has_bot_admin_rights(m.to_owned().sender.login, &crate::CONFIG)
            {
                if m.message_text.starts_with("!ping") {
                    tokio::spawn(async move { ping::handle(m, irc_client).await });
                }
                /*else if m.message_text.starts_with("!") {
                    tokio::spawn(async move { ::handle(m, irc_client).await });
                }*/
                /* else if m.message_text.starts_with("!") {
                    tokio::spawn(async move { ::handle(m, irc_client).await });
                }*/
            };
        },
        ServerMessage::Whisper { .. } => {
            let m = WhisperMessage::try_from(Into::<IRCMessage>::into(message.clone())).unwrap();
            // debug!("{:?}", &m);
            if has_bot_admin_rights(m.to_owned().sender.login, &crate::CONFIG) {
                #[allow(clippy::suspicious_else_formatting)]
                if m.message_text.starts_with("!link") {
                    tokio::spawn(async move { link::handle(m, irc_client).await });
                }
                /* else if m.message_text.starts_with("!") {
                    tokio::spawn(async move { ::handle(m, irc_client).await });
                }*/
                else {
                    super::parse_message("debug", format!("{:?}", message));
                }
            }
        },
        // All other cases are bunk
        _ => panic!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use twitch_irc::{
        login::RefreshingLoginCredentials,
        message::{IRCMessage, PrivmsgMessage, ServerMessage},
        ClientConfig, TwitchIRCClient,
    };

    const SRC: &str = "@badge-info=;badges=moderator/1;color=#AA66FF;display-name=TestUser;emotes=;flags=;id=8da29c58-d182-40cd-8b65-1dc446b45c65;mod=1;room-id=78127347;subscriber=0;tmi-sent-ts=1693037683123;turbo=0;user-id=12345678;user-type= :testuser!testuser@testuser.tmi.twitch.tv PRIVMSG #zoes17 :This is a test";

    fn get_irc_msg(msg: &str) -> IRCMessage {
        IRCMessage::parse(msg).unwrap()
    }

    fn get_privmsg(msg: IRCMessage) -> PrivmsgMessage {
        PrivmsgMessage::try_from(msg).unwrap()
    }

    fn get_svrmsg(msg: IRCMessage) -> ServerMessage {
        ServerMessage::try_from(msg).unwrap()
    }

    #[tokio::test]
    async fn parse() {
        let irc_message = get_irc_msg(SRC);
        let server_message = get_svrmsg(irc_message);
        let bts = BotTokenStorage::new();
        let rlc = RefreshingLoginCredentials::init_with_username(
            Some("TestUser".to_string()),
            "client_id".to_string(),
            "client_secret".to_string(),
            bts,
        );
        let client_config = ClientConfig::new_simple(rlc);
        let (_, client) = TwitchIRCClient::new(client_config);
        let t = parse_command(server_message, client).await;
        assert_eq!((), t);
    }

    #[test]
    fn mod_status() {
        let irc_message = get_irc_msg(SRC);
        let message = get_privmsg(irc_message);
        assert_eq!(true, has_mod_rights(message));
    }
}
