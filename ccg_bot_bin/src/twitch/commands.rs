use twitch_irc::login::RefreshingLoginCredentials;
use twitch_irc::message::{Badge, IRCMessage, PrivmsgMessage, ServerMessage};
use twitch_irc::{transport::tcp::TCPTransport, TwitchIRCClient};

use super::tokens::BotTokenStorage;

//command each in a module
mod ping;

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

pub async fn parse_command(
    message: ServerMessage,
    irc_client: TwitchIRCClient<
        TCPTransport<twitch_irc::transport::tcp::TLS>,
        RefreshingLoginCredentials<BotTokenStorage>,
    >,
) {
    match message {
        // Use for ban, timeout
        ServerMessage::ClearChat { .. } => {},
        // If we extend to monitoring deleted messages
        ServerMessage::ClearMsg { .. } => {},
        // pseudo-default case
        ServerMessage::Privmsg { .. } => {
            let m = PrivmsgMessage::try_from(Into::<IRCMessage>::into(message.clone())).unwrap();
            if has_mod_rights(m.to_owned()) && m.message_text.starts_with("!ping") {
                tokio::spawn(async move { ping::handle(m, irc_client).await });
            };
        },
        // All other cases are bunk
        _ => panic!(),
    }
}
