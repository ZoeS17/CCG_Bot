use async_trait::async_trait;
use chrono::serde::ts_seconds::deserialize as from_ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

//twitch_api
use twitch_api::helix::users::get_users::User;
#[cfg(not(test))]
use twitch_api::HelixClient;

//twitch_irc
use twitch_irc::{
    login::{GetAccessTokenResponse, RefreshingLoginCredentials, TokenStorage, UserAccessToken},
    ClientConfig,
};

//twitch_api
use twitch_api::twitch_oauth2::{
    client::Client,
    // scopes::Scope as ApiScope,
    scopes::Scope,
    tokens::{errors::RefreshTokenError, AppAccessToken, BearerTokenType},
    types::{AccessToken, ClientId, ClientSecret, RefreshToken},
    // types::{ClientId, ClientSecret},
    // TwitchToken as TwitchApiToken,
    TwitchToken,
};

//twitch_oauth2
// use twitch_oauth2::{Scope, TwitchToken};

//twitch_types
use twitch_types::{UserId, UserIdRef, UserName, UserNameRef};

// crate
use crate::{config::Config, utils::non_op_trace};
//skip reordering to allow easy reference to verbosity(from least to most)
#[rustfmt::skip]
use crate::{debug, trace};
#[cfg(not(test))]
use crate::twitch::api;
use crate::utils::approx_instant;

//std
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BotTokenStorage {
    pub prefix: String,
}

#[derive(Clone, Deserialize)]
#[cfg_attr(not(sensitive), derive(Debug))]
pub(crate) struct AppToken {
    pub access_token: twitch_oauth2::AccessToken,
    pub refresh_token: Option<twitch_oauth2::RefreshToken>,
    expires_in: Duration,
    #[serde(with = "approx_instant")]
    struct_created: Instant,
    client_id: ClientId,
    client_secret: ClientSecret,
    scopes: Vec<Scope>,
}

impl AppToken {
    #[allow(unused)]
    pub async fn new() -> Self {
        // Setup the http client to use with the library.
        let reqwest =
            reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();

        let config = crate::CONFIG.clone();

        // Grab the client id, convert to a `ClientId` with the `new` method.
        let client_id = twitch_oauth2::ClientId::new(config.twitch_client_id);
        let client_secret = twitch_oauth2::ClientSecret::new(config.twitch_client_secret);

        // Get the app access token
        let token = twitch_oauth2::AppAccessToken::get_app_access_token(
            &reqwest,
            client_id,
            client_secret,
            vec![],
        )
        .await
        .unwrap();
        token.into()
    }

    fn expires_in(&self) -> Duration {
        self.expires_in.checked_sub(self.struct_created.elapsed()).unwrap_or_default()
    }
}

#[async_trait]
impl TwitchToken for AppToken {
    fn token_type() -> BearerTokenType {
        BearerTokenType::AppAccessToken
    }

    fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    fn token(&self) -> &AccessToken {
        &self.access_token
    }

    fn login(&self) -> Option<&UserNameRef> {
        None
    }

    fn user_id(&self) -> Option<&UserIdRef> {
        None
    }

    async fn refresh_token<'a, C>(
        &mut self,
        http_client: &'a C,
    ) -> Result<(), RefreshTokenError<<C as Client>::Error>>
    where
        C: Client,
    {
        let (access_token, expires_in, refresh_token) =
            if let Some(token) = self.refresh_token.take() {
                token.refresh_token(http_client, &self.client_id, &self.client_secret).await?
            } else {
                return Err(RefreshTokenError::NoRefreshToken);
            };
        self.access_token = access_token;
        self.expires_in = expires_in;
        self.refresh_token = refresh_token;
        Ok(())
    }

    fn expires_in(&self) -> std::time::Duration {
        self.expires_in()
    }

    fn scopes(&self) -> &[Scope] {
        self.scopes.as_slice()
    }
}

#[cfg(sensitive)]
impl std::fmt::Debug for AppToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppToken")
            .field("access_token", &self.access_token.secret())
            .field("refresh_token", &self.refresh_token.as_ref().expect("").secret())
            .field("expires_in", &self.expires_in)
            .field("struct_created", &self.struct_created)
            .field("client_id", &self.client_id)
            .field("client_secret", &self.client_secret)
            .field("scopes", &self.scopes)
            .finish()
    }
}

impl From<AppAccessToken> for AppToken {
    fn from(value: AppAccessToken) -> Self {
        let expires_in = value.expires_in();
        let struct_created = Instant::now();
        let config = crate::CONFIG.clone();
        let client_id = ClientId::new(config.twitch_client_id.clone());
        let client_secret = ClientSecret::new(config.twitch_client_secret.clone());
        let scopes = super::api::SCOPE.to_vec();
        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            expires_in,
            struct_created,
            client_id,
            client_secret,
            scopes,
        }
    }
}

// impl From<twitch_oauth2::tokens::AppAccessToken> for AppToken {
//     fn from(value: twitch_oauth2::tokens::AppAccessToken) -> Self {
//         let expires_in = value.expires_in();
//         let struct_created = Instant::now();
//         let config = crate::CONFIG.clone();
//         let client_id = ClientId::new(config.twitch_client_id.clone());
//         let client_secret = ClientSecret::new(config.twitch_client_secret.clone());
//         let scopes = super::api::SCOPE.to_vec();
//         Self {
//             access_token: value.access_token,
//             refresh_token: value.refresh_token,
//             expires_in,
//             struct_created,
//             client_id,
//             client_secret,
//             scopes,
//         }
//     }
// }

#[derive(Clone, Deserialize)]
#[cfg_attr(not(sensitive), derive(Debug))]
pub(crate) struct Token {
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
    #[serde(deserialize_with = "from_ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(deserialize_with = "from_ts_seconds")]
    pub expires_at: DateTime<Utc>,
    /// When this struct was created, not when token was created.
    #[serde(with = "approx_instant")]
    pub struct_created: Instant,
    /// Expiration from when the response was generated.
    pub expires_in: Duration,
    pub clientid: ClientId,
    pub uid: UserId,
    pub name: UserName,
    pub(crate) client_secret: ClientSecret,
}

impl Token {
    fn expires_in(&self) -> Duration {
        self.expires_in.checked_sub(self.struct_created.elapsed()).unwrap_or_default()
    }

    #[allow(unused)]
    pub fn is_elapsed(&self) -> bool {
        let exp = self.expires_in();
        exp.as_secs() == 0 && exp.as_nanos() == 0
    }

    pub async fn set_uid(&mut self) -> Self {
        #[cfg(not(test))]
        let name = &self.name;
        #[cfg(not(test))]
        let client: HelixClient<'static, reqwest::Client> = HelixClient::default();
        #[cfg(not(test))]
        let user: Option<User> = client.get_user_from_login(name, &*self).await.unwrap();
        #[cfg(test)]
        let user: Option<User> = {
            use twitch_api::helix::{Request, RequestGet};
            let ids: &[&UserIdRef] = &["44322889".into()];
            let req = twitch_api::helix::users::GetUsersRequest::ids(ids);
            let data = br#"
{
    "data": [
        {
        "id": "141981764",
        "login": "twitchdev",
        "display_name": "TwitchDev",
        "type": "",
        "broadcaster_type": "partner",
        "description": "Supporting third-party developers building Twitch integrations from chatbots to game integrations.",
        "profile_image_url": "https://static-cdn.jtvnw.net/jtv_user_pictures/8a6381c7-d0c0-4576-b179-38bd5ce1d6af-profile_image-300x300.png",
        "offline_image_url": "https://static-cdn.jtvnw.net/jtv_user_pictures/3f13ab61-ec78-4fe6-8481-8682cb3b0ac2-channel_offline_image-1920x1080.png",
        "view_count": 5980557,
        "email": "not-real@email.com",
        "created_at": "2016-12-14T20:32:28.894263Z"
        }
    ]
    }
"#
        .to_vec();
            let http_response = rocket::http::hyper::Response::builder().body(data).unwrap();
            let uri = req.get_uri().unwrap();
            twitch_api::helix::users::GetUsersRequest::parse_response(
                Some(req),
                &uri,
                http_response,
            )
            .unwrap()
            .data
            .first()
            .cloned()
        };
        self.uid = user.unwrap().id;
        self.clone()
    }

    pub fn block_set_uid(&mut self) {
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async move {
                self.set_uid().await;
            });
        });
    }
}

#[async_trait]
impl TwitchToken for Token {
    fn token_type() -> BearerTokenType {
        BearerTokenType::UserToken
    }

    fn client_id(&self) -> &ClientId {
        &self.clientid
    }

    fn token(&self) -> &AccessToken {
        &self.access_token
    }

    fn login(&self) -> Option<&UserNameRef> {
        Some(&self.name)
    }

    fn user_id(&self) -> Option<&UserIdRef> {
        Some(&self.uid)
    }

    async fn refresh_token<'a, C>(
        &mut self,
        http_client: &'a C,
    ) -> Result<(), RefreshTokenError<<C as Client>::Error>>
    where
        Self: Sized,
        C: Client,
    {
        let (access_token, expires, refresh_token) = self
            .refresh_token
            .refresh_token(http_client, &self.clientid, &self.client_secret)
            .await?;
        self.access_token = access_token;
        self.expires_in = expires;
        self.refresh_token = refresh_token.expect("Never received a refresh token");
        Ok(())
    }

    fn expires_in(&self) -> Duration {
        self.expires_in()
    }

    fn scopes(&self) -> &[Scope] {
        &super::api::SCOPE
    }
}

#[cfg(sensitive)]
impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("access_token", &self.access_token.secret())
            .field("refresh_token", &self.refresh_token.secret())
            .field("created_at", &self.created_at)
            .field("expires_at", &self.expires_at)
            .field("struct_created", &self.struct_created)
            .field("expires_in", &self.expires_in)
            .field("client_id", &self.clientid)
            .field("uid", &self.uid.clone().take())
            .field("name", &self.name.clone().take())
            .field("client_secret", &self.client_secret)
            .finish()
    }
}

impl BotTokenStorage {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self { prefix: "TWITCH".to_string() }
    }

    pub fn init(&mut self, prefix: String) -> BotTokenStorage {
        if prefix.contains('_') {
            panic!("Prefix cannot contain underscores");
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
        let (initial_token, api_handle) =
            api::new(client_id.clone(), client_secret.clone(), redirect_url)
                .await
                .expect("Unable to generate an initial UserAccessToken");
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
        #[cfg(not(test))]
        api_handle.abort();
        #[cfg(not(test))]
        debug_assert!(non_op_trace(format!("{:?}", api_handle.await)));
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
        let key = format!("{}_{}", self.prefix, key);

        let value = std::env::var(&key).map_err(super::TwitchErr::VarErr)?;
        // Don't dump twitch vars here, see the end of ['load_token'] if this is actually needed
        if !key.starts_with("TWITCH") {
            trace!("Found env: {}={}", key, value);
        }

        let value = value.parse::<T>().map_err(|e| super::TwitchErr::FailedToParse {
            key,
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
        format!("{}_{}", self.prefix, key)
    }

    pub async fn token(&mut self) -> Token {
        self.load_token().await.unwrap().into()
    }
}

impl From<UserAccessToken> for Token {
    fn from(value: UserAccessToken) -> Self {
        let struct_created = Instant::now();
        let now = Utc::now();
        let expires_in = Duration::from_millis(
            (value.expires_at.expect("This should not be none in except in weird circumstances")
                - now)
                .num_milliseconds() as u64,
        );
        let config = crate::CONFIG.clone();
        let mut token = Self {
            access_token: AccessToken::new(value.access_token),
            refresh_token: RefreshToken::new(value.refresh_token),
            created_at: value.created_at,
            expires_at: value
                .expires_at
                .expect("This should not be none in except in weird circumstances"),
            struct_created,
            expires_in,
            clientid: ClientId::new(config.twitch_client_id.clone()),
            uid: UserId::from_static(""),
            name: UserName::new(config.twitch_bot_name.clone()),
            client_secret: ClientSecret::new(config.twitch_client_secret.clone()),
        };
        token.block_set_uid();
        token
    }
}

impl From<GetAccessTokenResponse> for Token {
    fn from(value: GetAccessTokenResponse) -> Self {
        let now = Utc::now();
        let expires_in =
            value.expires_in.expect("This should not be none in except in weird circumstances");
        let config = crate::CONFIG.clone();
        let mut token = Self {
            access_token: AccessToken::new(value.access_token),
            refresh_token: RefreshToken::new(value.refresh_token),
            // this is a bold-faced lie but isn't returned to us by Twitch
            created_at: now,
            expires_at: value.expires_in.map(|d| now + Duration::from_secs(d)).unwrap(),
            expires_in: Duration::from_secs(expires_in),
            struct_created: Instant::now(),
            clientid: ClientId::new(config.twitch_client_id.clone()),
            uid: UserId::from_static(""),
            name: UserName::new(config.twitch_bot_name.clone()),
            client_secret: ClientSecret::new(config.twitch_client_secret.clone()),
        };
        token.block_set_uid();
        token
    }
}

#[async_trait]
impl TokenStorage for BotTokenStorage {
    type LoadError = super::TwitchErr;
    type UpdateError = super::TwitchErr;

    async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
        let now = Utc::now();
        let at = match self.get_env("ACCESS_TOKEN") {
            Ok(v) => v,
            Err(e) => {
                debug!("[AccessToken]: {e}");
                "".to_string()
            },
        };
        let rt = match self.get_env("REFRESH_TOKEN") {
            Ok(v) => v,
            Err(e) => {
                debug!("[RefreshToken]: {e}");
                "".to_string()
            },
        };
        let ca: DateTime<Utc> = match self.get_env("TOKEN_CREATED_AT") {
            Ok(v) => v,
            Err(e) => {
                debug!("[CreatedAt]: {e}");
                now
            },
        };
        let ea: DateTime<Utc> = match self.get_env("TOKEN_EXPIRES_AT") {
            Ok(v) => v,
            Err(e) => {
                debug!("[ExpiresAt]: {e}");
                // 7500 seconds is 2 hours 5 minutes
                now + Duration::from_secs(7500_u64)
            },
        };
        let ei: Duration = match self.get_env("TOKEN_EXPIRES_IN") {
            Ok(v) => Duration::from_secs(v),
            Err(e) => {
                debug!("[ExpiresIn]: {e}");
                let sys_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("").as_secs();
                if ea.timestamp()
                    > TryInto::<i64>::try_into(sys_now).expect("SystemTime was a negative i64")
                {
                    Duration::from_secs((ea.timestamp() as u64) - sys_now)
                } else {
                    Duration::from_secs(0_u64)
                }
            },
        };
        let uid: UserId = match self.get_env("USER_ID") {
            Ok(v) => UserId::new(v),
            Err(e) => {
                debug!("[UserId]: {e}");
                UserId::new(String::new())
            },
        };
        let name: UserName = match self.get_env("USERNAME") {
            Ok(v) => UserName::new(v),
            Err(e) => {
                debug!("[UserName]: {e}");
                UserName::new(String::new())
            },
        };
        let config = crate::CONFIG.clone();
        let token = Token {
            access_token: AccessToken::new(at),
            refresh_token: RefreshToken::new(rt),
            created_at: ca,
            expires_at: ea,
            struct_created: Instant::now(),
            expires_in: ei,
            clientid: ClientId::new(config.twitch_client_id.clone()),
            uid,
            name,
            client_secret: ClientSecret::new(config.twitch_client_secret.clone()),
        };
        let uat = UserAccessToken {
            access_token: token.access_token.clone().take(),
            refresh_token: token.refresh_token.clone().take(),
            created_at: token.created_at,
            expires_at: Some(token.expires_at),
        };
        // Put this behind trace and a feature so that nobody accidentally gives us their access token(s)
        #[cfg(sensitive)]
        debug_assert!(non_op_trace(format!("[load_token] token = {token:?}")));
        debug_assert!(non_op_trace(format!("uat {:?}", &uat)));
        Ok(uat)
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
        self.set_env("ACCESS_TOKEN", &token.access_token).unwrap();
        self.set_env("REFRESH_TOKEN", &token.refresh_token).unwrap();
        self.set_env("TOKEN_CREATED_AT", token.created_at).unwrap();
        self.set_env("TOKEN_EXPIRES_AT", token.expires_at.unwrap_or_default()).unwrap();
        let config = crate::CONFIG.clone();
        self.set_env("CLIENT_ID", config.twitch_client_id.clone()).unwrap();
        self.set_env("USERNAME", config.twitch_bot_name.clone()).unwrap();
        let mut token: Token = self.token().await;
        token.block_set_uid();
        self.set_env("USER_ID", token.uid.clone().take()).unwrap();
        // Make this trace(in unoptimized builds only) so that nobody accidentally gives us their access token(s)
        // but so if needed due to any upstream changes unknown errors can be easier to debug
        debug_assert!(non_op_trace(format!("[update_token] token = {token:?}")));
        Ok(())
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn app_token() {
        let config = crate::CONFIG.clone();
        let access_token = twitch_oauth2::AccessToken::new(String::from("TestAccessToken"));
        let refresh_token = Some(twitch_oauth2::RefreshToken::new(String::from("TestRefreshToken")));
        let app_token = AppToken {
            access_token,
            refresh_token,
            expires_in: Duration::new(30,0),
            struct_created: Instant::now(),
            client_id: ClientId::new(config.twitch_client_id),
            client_secret: ClientSecret::new(config.twitch_client_secret),
            scopes: vec![],
        };
        let app_token_str = r#"
        {
            "access_token": "TestAccessToken",
            "refresh_token": "TestRefreshToken",
            "expires_in": {
                "secs": 30,
                "nanos": 0
            },
            "struct_created": {
                "secs_since_epoch": 1728195721,
                "nanos_since_epoch": 237168157
            },
            "client_id": "IamAclientId",
            "client_secret": "IamAclientSecret",
            "scopes": []
        }"#;
        let _: AppToken = serde_json::from_str(&app_token_str).unwrap();
        let _ = AppToken::token_type();
        let _ = AppToken::client_id(&app_token);
        let _ = AppToken::token(&app_token);
        let _ = AppToken::login(&app_token);
        let _ = AppToken::user_id(&app_token);
        let _ = AppToken::expires_in(&app_token);
        let _ = AppToken::scopes(&app_token);
    }
}