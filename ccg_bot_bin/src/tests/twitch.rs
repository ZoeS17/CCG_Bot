use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use twitch_irc::login::UserAccessToken;

#[derive(Default, Deserialize, Serialize)]
pub struct TestAccessToken {
    pub access_token: String,
    pub refresh_token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<UserAccessToken> for TestAccessToken {
    fn from(value: UserAccessToken) -> Self {
        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            created_at: value.created_at,
            expires_at: value.expires_at,
        }
    }
}

#[test]
fn it_works() {
    use super::super::config::Config;
    use super::super::twitch::{new, Handler};
    let twitch: Result<Handler, std::env::VarError> = aw!(new(Config {
        #[cfg(any(feature = "discord", feature = "full"))]
        discord_guildid: "".to_string(),
        #[cfg(any(feature = "discord", feature = "full"))]
        discord_token: "".to_string(),
        #[cfg(any(feature = "twitch", feature = "full"))]
        twitch_bot_name: "".to_string(),
        #[cfg(any(feature = "twitch", feature = "full"))]
        twitch_channels: vec!["".to_string()],
        #[cfg(any(feature = "twitch", feature = "full"))]
        twitch_client_id: "".to_string(),
        #[cfg(any(feature = "twitch", feature = "full"))]
        twitch_client_secret: "".to_string(),
        #[cfg(any(feature = "twitch", feature = "full"))]
        twitch_redirect_url: "http://localhost/".to_string()
    }));
    let twitch_bool: bool = twitch.is_ok();
    assert!(twitch_bool);
}
