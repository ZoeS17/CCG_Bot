use async_trait::async_trait;
use chrono::serde::ts_seconds::deserialize as from_ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

//twitc_irc
use twitch_irc::{
    login::{GetAccessTokenResponse, RefreshingLoginCredentials, TokenStorage, UserAccessToken},
    ClientConfig,
};

//twtich_oauth2
#[cfg(not(test))]
use twitch_oauth2::{ClientId, ClientSecret};

// crate
use crate::config::Config;
//skip reordering to allow easy reference to verbosity(from least to most)
#[rustfmt::skip]
use crate::trace;
#[cfg(not(test))]
use crate::twitch::api;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct BotTokenStorage {
    prefix: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct Token {
    access_token: String,
    refresh_token: String,
    #[serde(deserialize_with = "from_ts_seconds")]
    created_at: DateTime<Utc>,
    #[serde(deserialize_with = "from_ts_seconds")]
    expires_at: DateTime<Utc>,
}

impl BotTokenStorage {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self { prefix: Some("TWITCH".to_string()) }
    }

    pub fn init(&mut self, prefix: Option<String>) -> BotTokenStorage {
        if let Some(prefix) = &prefix {
            if prefix.contains('_') {
                panic!("Prefix cannot contain underscores");
            }
        }
        Self { prefix }
    }

    pub async fn client_config(
        mut self,
        config: Config,
    ) -> ClientConfig<RefreshingLoginCredentials<BotTokenStorage>> {
        let username = Some(config.twitch_bot_name);
        let client_id = config.twitch_client_id;
        let client_secret = config.twitch_client_secret;
        #[cfg(not(test))]
        let redirect_url = config.twitch_redirect_url;
        #[cfg(not(test))]
        let (initial_token, _api_handle) = api::new(
            ClientId::new(client_id.clone()),
            ClientSecret::new(client_secret.clone()),
            redirect_url,
        )
        .await
        .expect("Failed to get UserAccessToken");
        #[cfg(test)]
        let test_initial_token_default = crate::tests::twitch::TestAccessToken::default();
        #[cfg(test)]
        let test_initial_token_str = serde_json::to_string(&test_initial_token_default).unwrap();
        #[cfg(test)]
        let initial_token =
            serde_json::from_str::<UserAccessToken>(&test_initial_token_str).unwrap();
        self.update_token(&initial_token).await.expect("");
        let env = self;
        let rlc =
            RefreshingLoginCredentials::init_with_username(username, client_id, client_secret, env);
        ClientConfig::new_simple(rlc)
    }

    pub fn get_env<T>(&self, key: &str) -> Result<T, super::TwitchErr>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        self.get_env_opt(key)?.ok_or(super::TwitchErr::VarErr(std::env::VarError::NotPresent))
    }

    pub fn get_env_opt<T>(&self, key: &str) -> Result<Option<T>, super::TwitchErr>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        let key = if let Some(prefix) = &self.prefix {
            format!("{}_{}", prefix, key)
        } else {
            key.to_string()
        };

        let value = std::env::var(&key).map_err(super::TwitchErr::VarErr)?;
        // Make this trace so that no accidently gives us their access token(s)
        // but so if needed due to any upstream changes unknown errors can be easier to debug
        trace!("Found env: {}={}", key, value);

        let value = value.parse::<T>().map_err(|e| super::TwitchErr::FailedToParse {
            key: key.to_string(),
            value: value.to_string(),
            error: Some(format!("{:?}", e)),
        })?;

        Ok(Some(value))
    }

    pub fn set_env<T>(&self, key: &str, value: T) -> Result<(), super::TwitchErr>
    where
        T: ToString,
    {
        let key = self.format_key(key);
        let value = value.to_string();
        std::env::set_var(key, value);

        Ok(())
    }

    fn format_key(&self, key: &str) -> String {
        let key = key.to_uppercase();
        if let Some(prefix) = &self.prefix {
            format!("{}_{}", prefix, key)
        } else {
            key
        }
    }
}

impl From<UserAccessToken> for Token {
    fn from(value: UserAccessToken) -> Self {
        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            created_at: value.created_at,
            expires_at: value.expires_at.unwrap_or_default(),
        }
    }
}

impl From<GetAccessTokenResponse> for Token {
    fn from(value: GetAccessTokenResponse) -> Self {
        let now = Utc::now();
        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            created_at: now,
            expires_at: value
                .expires_in
                .map(|d| now + chrono::Duration::from_std(Duration::from_secs(d)).unwrap())
                .unwrap(),
        }
    }
}

#[async_trait]
impl TokenStorage for BotTokenStorage {
    type LoadError = std::io::Error; // or some other error
    type UpdateError = std::io::Error;

    async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
        let at = match self.get_env("ACCESS_TOKEN") {
            Ok(v) => v,
            Err(_) => "".to_string(),
        };
        let rt = match self.get_env("REFRESH_TOKEN") {
            Ok(v) => v,
            Err(_) => "".to_string(),
        };
        let ca: DateTime<Utc> = match self.get_env("TOKEN_CREATED_AT") {
            Ok(v) => v,
            Err(_) => Utc::now(),
        };
        let ea: DateTime<Utc> = match self.get_env("TOKEN_EXPIRES_AT") {
            Ok(v) => v,
            Err(_) => Utc::now() + chrono::Duration::seconds(7500_i64),
        };
        let token = Token { access_token: at, refresh_token: rt, created_at: ca, expires_at: ea };
        // Make this trace so that no accidently gives us their access token(s)
        // but so if needed due to any upstream changes unknown errors can be easier to debug
        trace!("[load_token] token = {token:?}");
        let uat = UserAccessToken {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            created_at: token.created_at,
            expires_at: Some(token.expires_at),
        };
        trace!("uat {:?}", &uat);
        Ok(uat)
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
        // Make this trace so that no accidently gives us their access token(s)
        // but so if needed due to any upstream changes unknown errors can be easier to debug
        trace!("[update_token] token = {token:?}");
        self.set_env("ACCESS_TOKEN", &token.access_token).unwrap();
        self.set_env("REFRESH_TOKEN", &token.refresh_token).unwrap();
        self.set_env("TOKEN_CREATED_AT", token.created_at).unwrap();
        self.set_env("TOKEN_EXPIRES_AT", token.expires_at.unwrap_or_default()).unwrap();
        Ok(())
    }
}
