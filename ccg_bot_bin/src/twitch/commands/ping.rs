//Example-ish command

use twitch_irc::{
    login::RefreshingLoginCredentials,
    message::PrivmsgMessage,
    transport::tcp::{TCPTransport, TLS},
    Error, TwitchIRCClient,
};

use super::BotTokenStorage;

pub async fn handle(
    message: PrivmsgMessage,
    client: TwitchIRCClient<TCPTransport<TLS>, RefreshingLoginCredentials<BotTokenStorage>>,
) -> Result<(), Error<TCPTransport<TLS>, RefreshingLoginCredentials<BotTokenStorage>>> {
    let reply_to = message;
    let response = String::from("pong");
    // It's unfortunate that the call to the say_in_reply_to function on client can't be tested directly
    #[cfg(not(test))]
    {
        client.say_in_reply_to(&reply_to, response).await
    }
    #[cfg(test)]
    {
        let _ = reply_to;
        let _ = response;
        let _ = message;
        let _ = client;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use twitch_irc::{
        login::RefreshingLoginCredentials,
        message::{IRCMessage, PrivmsgMessage},
        ClientConfig, TwitchIRCClient,
    };

    use super::handle;
    use super::BotTokenStorage;
    // use crate::tests::aw;

    #[tokio::test]
    async fn command_handle() {
        let src = "@badge-info=;badges=;color=#AA66FF;display-name=TestUser;emotes=;flags=;id=8da29c58-d182-40cd-8b65-1dc446b45c65;mod=1;room-id=78127347;subscriber=1;tmi-sent-ts=1693037683123;turbo=1;user-id=12345678;user-type= :testuser!testuser@testuser.tmi.twitch.tv PRIVMSG #zoes17 :This is a test";
        let irc_message = IRCMessage::parse(src).unwrap();
        let message = PrivmsgMessage::try_from(irc_message).unwrap();

        let bts = BotTokenStorage::new();
        let rlc = RefreshingLoginCredentials::init_with_username(
            Some("TestUser".to_string()),
            "client_id".to_string(),
            "client_secret".to_string(),
            bts,
        );
        let client_config = ClientConfig::new_simple(rlc);
        let (_, client) = TwitchIRCClient::new(client_config);

        let t = handle(message, client).await.ok().expect("This should always be unit anyway");
        let expected = ();
        assert_eq!(expected, t);
    }
}
