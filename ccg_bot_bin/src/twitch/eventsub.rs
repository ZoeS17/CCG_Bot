//skip reordering to allow easy reference to verbosity(from least to most)
#[rustfmt::skip]
use crate::{error, warn, info/*, info_span */,debug, trace};

use eyre::Context;
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::tungstenite;
use twitch_api::{
    eventsub::{
        self,
        event::websocket::{EventsubWebsocketData, ReconnectPayload, SessionData, WelcomePayload},
        Event,
    },
    twitch_oauth2::TwitchToken,
    types, HelixClient,
};
use twitch_types::UserName;

use crate::{
    twitch::tokens::{AppToken, Token},
    utils::non_op_dbg,
};

// WebSockets use user access tokens
#[allow(unused)]
pub struct WebsocketClient {
    /// The session id of the websocket connection
    pub session_id: Option<String>,
    /// The UserToken used to authenticate with the Twitch API
    pub user_token: Token,
    /// The AppToken used to authenticate with the Twitch API
    pub app_token: AppToken,
    /// The client used to make requests to the Twitch API
    pub client: HelixClient<'static, reqwest::Client>,
    /// The user id of the channel we want to listen to
    pub user_id: types::UserId,
    /// The url to use for websocket
    pub connect_url: tungstenite::http::Uri,
}

impl WebsocketClient {
    /// Connect to the websocket and return the stream
    #[allow(unused)]
    pub async fn connect(
        &self,
    ) -> Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        eyre::Error,
    > {
        info!("connecting to twitch");
        let config = tungstenite::protocol::WebSocketConfig {
            max_write_buffer_size: 9 << 14,   // 18 KiB
            max_message_size: Some(64 << 20), // 64 MiB
            max_frame_size: Some(16 << 20),   // 16 MiB
            accept_unmasked_frames: false,
            ..tungstenite::protocol::WebSocketConfig::default()
        };
        let (socket, _) = tokio_tungstenite::connect_async_tls_with_config(
            &self.connect_url,
            Some(config),
            false,
            None,
        )
        .await
        .context("Can't connect")?;
        Ok(socket)
    }

    /// Run the websocket subscriber
    #[allow(unused)]
    pub async fn run(mut self) -> eyre::Result<()> {
        // Establish the stream
        let mut s = self.connect().await.context("when establishing connection")?;
        // Loop over the stream, processing messages as they come in.
        loop {
            tokio::select!(
            Some(msg) = futures::StreamExt::next(&mut s) => {
                let msg = match msg {
                    Err(tungstenite::Error::Protocol(
                        tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
                    )) => {
                        warn!(
                            "connection was sent an unexpected frame or was reset, reestablishing it"
                        );
                        s = self
                            .connect()
                            .await
                            .context("when reestablishing connection")?;
                        continue
                    }
                    _ => {
                        if msg.is_err() {
                            debug_assert!(non_op_dbg(format!("{:?}", &msg)));
                        }
                        msg.context("when getting message")?
                    },
                };
                self.process_message(&mut s, msg).await?
            })
        }
    }

    /// Process a message from the websocket
    #[allow(unused)]
    pub async fn process_message(
        &mut self,
        sock: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        msg: tungstenite::Message,
    ) -> eyre::Result<()> {
        match msg {
            tungstenite::Message::Text(s) => {
                // Parse the message into a [twitch_api::eventsub::EventsubWebsocketData]
                let parsed_msg = Event::parse_websocket(&s)?;
                match parsed_msg {
                    EventsubWebsocketData::Welcome {
                        payload: WelcomePayload { session }, ..
                    }
                    | EventsubWebsocketData::Reconnect {
                        payload: ReconnectPayload { session },
                        ..
                    } => {
                        self.process_welcome_message(session).await?;
                        Ok(())
                    },
                    // Here is where you would handle the events you want to listen to
                    EventsubWebsocketData::Notification { metadata: _, payload } => {
                        match payload {
                            Event::ChannelBanV1(eventsub::Payload { message, .. }) => {
                                let m = message;
                                info!(?m, "got ban event");
                            },
                            Event::ChannelUnbanV1(eventsub::Payload { message, .. }) => {
                                let m = message;
                                info!(?m, "got ban event");
                            },
                            _ => {},
                        }
                        Ok(())
                    },
                    EventsubWebsocketData::Revocation { metadata, payload: _ } => {
                        eyre::bail!("got revocation event: {metadata:?}")
                    },
                    EventsubWebsocketData::Keepalive { metadata, payload: _ } => {
                        Ok(trace!("Staying alive: {metadata:?}"))
                    },
                    _ => Ok(()),
                }
            },
            tungstenite::Message::Close(ocf) => match ocf {
                Some(cf) => Err(eyre::eyre!("Socket closed [{:?}] with: {}", cf.code, cf.reason)),
                None => Ok(()),
            },
            tungstenite::Message::Binary(vu8) => Ok(debug!("received binary message: {vu8:?}")),
            tungstenite::Message::Ping(vu8) => {
                let len = sock.get_mut().write(&vu8).await?;
                Ok(trace!("received ping with length: {len}"))
            },
            tungstenite::Message::Pong(vu8) => {
                let len = vu8.len();
                Ok(debug!("received pong with length: {len}"))
            },
            tungstenite::Message::Frame(f) => Ok(debug!("Raw frame: {f}")),
        }
    }

    #[allow(unused)]
    pub async fn process_welcome_message(
        &mut self,
        data: SessionData<'_>,
    ) -> Result<(), eyre::Report> {
        self.session_id = Some(data.id.to_string());
        if let Some(url) = data.reconnect_url {
            self.connect_url = url.parse()?;
        }
        // check if the token is expired, if it is, request a new token. This only works if using a oauth service for getting a token
        if self.user_token.is_elapsed() {
            self.user_token.refresh_token(&self.client).await?;
        }
        let transport = eventsub::Transport::websocket(data.id.clone());
        let channels: Vec<String> = crate::CONFIG.clone().twitch_channels;
        let bot_name = crate::CONFIG.clone().twitch_bot_name;
        assert_eq!(self.user_token.name.clone().take(), bot_name);
        for channel in channels {
            if channel.as_str().to_lowercase() == bot_name {
                let user_id = self
                    .client
                    .get_user_from_login(&UserName::new(channel.clone()), &self.user_token)
                    .await?
                    .unwrap_or_else(|| panic!("Unable to retrieve user from: {}", channel.clone()))
                    .id;
                match self
                    .client
                    .create_eventsub_subscription(
                        eventsub::channel::ChannelBanV1::broadcaster_user_id(user_id.clone()),
                        transport.clone(),
                        &self.user_token,
                    )
                    .await
                {
                    Ok(i) => info!("[{}] {}", channel.clone(), i.condition.broadcaster_user_id),
                    Err(e) => error!("[{}] {:?}", channel.clone(), e),
                };
                match self
                    .client
                    .create_eventsub_subscription(
                        eventsub::channel::ChannelUnbanV1::broadcaster_user_id(user_id),
                        transport.clone(),
                        &self.user_token,
                    )
                    .await
                {
                    Ok(i) => debug!("[{}] {}", channel.clone(), i.condition.broadcaster_user_id),
                    Err(e) => error!("[{}] {:?}", channel.clone(), e),
                };
            } else {
                //TODO: Add logic to get a user token from users that aren't the bot itself.
                info!("ignoring channel `{channel}` as non-bot users aren't supported yet.");
            }
        }
        info!("listening to ban and unbans");
        Ok(())
    }
}
