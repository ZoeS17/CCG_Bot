//Example-ish command

use twitch_irc::{
    Error, TwitchIRCClient,
    login::RefreshingLoginCredentials,
    message::PrivmsgMessage,
    transport::tcp::{TCPTransport, TLS}
};

use super::BotTokenStorage;

pub async fn handle(message: PrivmsgMessage, client: TwitchIRCClient<TCPTransport<TLS>, RefreshingLoginCredentials<BotTokenStorage>>)
    -> Result<(), Error<TCPTransport<TLS>, RefreshingLoginCredentials<BotTokenStorage>>>
{
    let reply_to = message;
    let response = String::from("pong");
    client.say_in_reply_to(&reply_to, response).await
}