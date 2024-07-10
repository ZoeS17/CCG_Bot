//discord-twitch link

#[rustfmt::skip]
use crate::{error, debug};

use twitch_irc::{
    login::RefreshingLoginCredentials,
    message::{IRCMessage, WhisperMessage},
    transport::tcp::{TCPTransport, TLS},
    Error, TwitchIRCClient,
};

use super::BotTokenStorage;

async fn send_message(
    client: TwitchIRCClient<TCPTransport<TLS>, RefreshingLoginCredentials<BotTokenStorage>>,
    message: IRCMessage,
) -> Result<(), Error<TCPTransport<TLS>, RefreshingLoginCredentials<BotTokenStorage>>> {
    #[cfg(test)]
    {
        let _ = client;
        let _ = message;
    }
    #[cfg(test)]
    let res = Ok(());
    #[cfg(not(test))]
    let res = client.send_message(message).await;
    res
}

pub async fn handle(
    message: WhisperMessage,
    client: TwitchIRCClient<TCPTransport<TLS>, RefreshingLoginCredentials<BotTokenStorage>>,
) -> Result<(), Error<TCPTransport<TLS>, RefreshingLoginCredentials<BotTokenStorage>>> {
    #[cfg(test)]
    let _ = client;
    // It's unfortunate that the call to the say_in_reply_to function on client can't be tested directly
    let bot_name = crate::CONFIG.clone().twitch_bot_name;
    let bot_id = match std::env::var("TWITCH_USER_ID") {
        Ok(i) => i,
        Err(_) => {
            error!("TWITCH_USER_ID not found in env defaulting to blank");
            String::from("")
        },
    };
    let reply_to = message.sender;
    let content = message.message_text;
    let parsed_content: Vec<String> = content.split_whitespace().map(|s| s.to_string()).collect();
    let (first, second): (String, String);
    let res: Result<(), Error<TCPTransport<TLS>, RefreshingLoginCredentials<BotTokenStorage>>>;
    if parsed_content.len() == 3 {
        (first, second) =
            (parsed_content.get(1).unwrap().clone(), parsed_content.get(2).unwrap().clone());
        let (twitch_un, discord_id): (String, String);
        if second.parse::<u64>().is_ok() {
            twitch_un = first;
            discord_id = second;
        } else {
            twitch_un = second;
            discord_id = first;
            let raw_interim_response = format!("@badges=;color=#AA66FF;display-name={};emotes=;message-id=1;thread-id={}_{};turbo=1;user-id=12345678;user-type= :{}!{}@{}.tmi.twitch.tv WHISPER {} :Assuming inverted params", reply_to.login, bot_id, reply_to.id, bot_name, bot_name, bot_name, reply_to.name);
            let interim_response = IRCMessage::parse(&raw_interim_response);
            send_message(client.clone(), interim_response.unwrap()).await?;
        }
        if !twitch_un.is_empty() && !discord_id.is_empty() {
            //TODO: Actually link these accounts in someway for now do this
            debug!("twitch_un={twitch_un:?} discord_id={discord_id:?}");
        } else {
            let (_, _) = (twitch_un, discord_id);
        }
        let raw_response = format!("@badges=;color=#AA66FF;display-name={};emotes=;message-id=1;thread-id={}_{};turbo=1;user-id=12345678;user-type= :{}!{}@{}.tmi.twitch.tv WHISPER {} :Understood", reply_to.login, bot_id, reply_to.id, bot_name, bot_name, bot_name, reply_to.name);
        let response = IRCMessage::parse(&raw_response);
        res = send_message(client, response.unwrap()).await;
    } else {
        let raw_response = format!("@badges=;color=#AA66FF;display-name={};emotes=;message-id=1;thread-id={}_{};turbo=1;user-id=12345678;user-type= :{}!{}@{}.tmi.twitch.tv WHISPER {} :[Usage] !link <twitch @> <discord id number>", reply_to.login, bot_id, reply_to.id, bot_name, bot_name, bot_name, reply_to.name);
        let response = IRCMessage::parse(&raw_response);
        res = send_message(client, response.unwrap()).await;
    }
    res
}

#[cfg(test)]
mod test {
    use super::*;
    use twitch_irc::ClientConfig;

    #[tokio::test]
    async fn command_handle() {
        // WHISPER from TestUser to UserTest
        let src = "@badges=;color=#AA66FF;display-name=TestUser;emotes=;message-id=1;thread-id=12345678_87654321;turbo=1;user-id=12345678;user-type= :testuser!testuser@testuser.tmi.twitch.tv WHISPER usertest :!link CourtesyCallGaming 379001295744532481";
        let irc_message = IRCMessage::parse(src).unwrap();
        let message = WhisperMessage::try_from(irc_message).unwrap();

        let bts = BotTokenStorage::new();
        let rlc = RefreshingLoginCredentials::init_with_username(
            Some("TestUser".to_string()),
            "client_id".to_string(),
            "client_secret".to_string(),
            bts.clone(),
        );
        let client_config = ClientConfig::new_simple(rlc);
        let (mut rx, client) = TwitchIRCClient::new(client_config);
        while !rx.is_empty() {
            let _ = dbg!(rx.recv().await);
        }
        let t = handle(message, client).await.unwrap();
        let expected = ();
        assert_eq!(expected, t);
    }

    #[tokio::test]
    async fn command_handle_flipped_params() {
        // WHISPER from TestUser to UserTest
        let src = "@badges=;color=#AA66FF;display-name=TestUser;emotes=;message-id=1;thread-id=12345678_87654321;turbo=1;user-id=12345678;user-type= :testuser!testuser@testuser.tmi.twitch.tv WHISPER usertest :!link 379001295744532481 CourtesyCallGaming";
        let irc_message = IRCMessage::parse(src).unwrap();
        let message = WhisperMessage::try_from(irc_message).unwrap();

        let bts = BotTokenStorage::new();
        let rlc = RefreshingLoginCredentials::init_with_username(
            Some("TestUser".to_string()),
            "client_id".to_string(),
            "client_secret".to_string(),
            bts.clone(),
        );
        let client_config = ClientConfig::new_simple(rlc);
        let (mut rx, client) = TwitchIRCClient::new(client_config);
        while !rx.is_empty() {
            let _ = dbg!(rx.recv().await);
        }
        let t = handle(message, client).await.unwrap();
        let expected = ();
        assert_eq!(expected, t);
    }

    #[tokio::test]
    async fn command_handle_too_few_params() {
        // WHISPER from TestUser to UserTest
        let src = "@badges=;color=#AA66FF;display-name=TestUser;emotes=;message-id=1;thread-id=12345678_87654321;turbo=1;user-id=12345678;user-type= :testuser!testuser@testuser.tmi.twitch.tv WHISPER usertest :!link CourtesyCallGaming";
        let irc_message = IRCMessage::parse(src).unwrap();
        let message = WhisperMessage::try_from(irc_message).unwrap();

        let bts = BotTokenStorage::new();
        let rlc = RefreshingLoginCredentials::init_with_username(
            Some("TestUser".to_string()),
            "client_id".to_string(),
            "client_secret".to_string(),
            bts.clone(),
        );
        let client_config = ClientConfig::new_simple(rlc);
        let (mut rx, client) = TwitchIRCClient::new(client_config);
        while !rx.is_empty() {
            let _ = dbg!(rx.recv().await);
        }
        let t = handle(message, client).await.unwrap();
        let expected = ();
        assert_eq!(expected, t);
    }
}
