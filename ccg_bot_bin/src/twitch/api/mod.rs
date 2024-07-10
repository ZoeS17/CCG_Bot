use reqwest::Url;
use rocket::{
    get, http::ContentType, response::Responder, routes, Ignite, Response, Rocket, Shutdown, State,
};
use serde::Serialize;
use std::{
    error::Error,
    fmt::Display,
    io::Cursor,
    sync::mpsc::{self, Receiver, SendError, Sender},
};
use tokio::task::{JoinError, JoinHandle};
#[cfg(test)]
use tokio::time;
use twitch_irc::login::{GetAccessTokenResponse, UserAccessToken};
use twitch_oauth2::{tokens::UserTokenBuilder, ClientId, ClientSecret, Scope};

pub const SCOPE: [Scope; 27] = [
    Scope::ChannelModerate,
    Scope::ChannelReadRedemptions,
    Scope::ChatRead,
    Scope::ChatEdit,
    Scope::WhispersRead,
    Scope::WhispersEdit,
    Scope::ChannelEditCommercial,
    Scope::ClipsEdit,
    Scope::ChannelManageBroadcast,
    Scope::UserReadBlockedUsers,
    Scope::UserManageBlockedUsers,
    Scope::ModeratorManageAutoMod,
    Scope::ChannelManageRaids,
    Scope::ChannelManagePolls,
    Scope::ChannelManagePredictions,
    Scope::ChannelReadPredictions,
    Scope::ModeratorManageAnnouncements,
    Scope::UserManageWhispers,
    Scope::ModeratorManageBannedUsers,
    Scope::ModeratorManageChatMessages,
    Scope::UserManageChatColor,
    Scope::ModeratorManageChatSettings,
    Scope::ChannelManageModerators,
    Scope::ChannelManageVips,
    Scope::ModeratorReadChatters,
    Scope::ModeratorManageShieldMode,
    Scope::ModeratorManageShoutouts,
];

#[allow(unused)]
pub async fn new(
    client_id: impl Into<ClientId>,
    client_secret: impl Into<ClientSecret>,
    redirect_url: String,
) -> Result<(UserAccessToken, JoinHandle<Result<Rocket<Ignite>, rocket::Error>>), Box<dyn Error>> {
    let mut token_builder =
        UserTokenBuilder::new(client_id, client_secret, Url::parse(&redirect_url).unwrap())
            .set_scopes(SCOPE.to_vec());
    // get url for auth page
    let (auth_page_url, _) = token_builder.generate_url();

    // initialize rocket and mount routes
    let (auth_tx, auth_rx) = mpsc::channel::<UserAccessToken>();
    let rocket = rocket::build()
        .manage(RocketAuthState { auth_tx, token_builder })
        .mount("/", routes![oauth_code_callback, catch_oauth_error])
        .ignite()
        .await?;

    // get a shutdown handle (to stop rocket after authentication) and launch rocket (no need to
    // await; awaiting will wait for the task to end)
    let shutdown_handle = rocket.shutdown();
    #[cfg(test)]
    let shutdown = shutdown_handle.clone();
    #[cfg(test)]
    let response = r#"{"access_token":"TestAccessToken", "refresh_token":"TestRefreshToken", "expires_in": 10, "scope": ["user_read"], "token_type": "bearer"}"#;
    #[cfg(test)]
    let user_access_token =
        UserAccessToken::from(serde_json::from_str::<GetAccessTokenResponse>(&response)?);
    #[cfg(test)]
    let maybe_state = rocket.state().cloned();
    #[cfg(test)]
    let auth_state: RocketAuthState = maybe_state.unwrap();
    #[cfg(test)]
    auth_state.auth_tx.send(user_access_token)?;
    #[cfg(test)]
    tokio::task::spawn(async {
        time::sleep(time::Duration::from_secs(30)).await;
        shutdown.notify();
    });
    let rocket_handle = launch_rocket(rocket);

    // open web browser to authorize
    // and wait for the task to end
    open_auth_page(auth_page_url);
    // wait for main task to end
    let token = get_token(auth_rx, shutdown_handle).await?;

    // wait for rocket execution to end
    //
    // one `?` is to check for a `JoinError` and the other is for checking for a rocket launch
    // error
    // let _ = rocket_handle.await??;

    Ok((token, rocket_handle))
}

async fn get_token(
    auth_rx: Receiver<UserAccessToken>,
    shutdown_handle: Shutdown,
) -> Result<UserAccessToken, JoinError> {
    tokio::task::spawn(async move {
        println!("waiting for auth token...");
        let token = auth_rx.recv().unwrap();
        println!("got auth token! shutting down server");

        // we have a token now, so we don't need to listen at our endpoints anymore
        shutdown_handle.notify();
        println!("server is shut down");
        token
    })
    .await
}

/// Opens the Twitch autorization page with a new thread. open-rs is not supposed to block, but it
/// does anyways for some reason lol
#[allow(unused)]
fn open_auth_page(auth_page_url: reqwest::Url) {
    println!("opening authorization page");
    #[cfg(not(test))]
    if let Err(e) = open::that_in_background(auth_page_url.to_string()).join() {
        eprintln!("couldn't open url: {e:?}");
        eprintln!("to authorize, open up this url: {auth_page_url}");
    } else {
        println!("opened auth page");
    }
}

/// Launches an ignited `Rocket` in a separate thread.
#[allow(unused)]
fn launch_rocket(
    rocket: Rocket<rocket::Ignite>,
) -> JoinHandle<Result<Rocket<Ignite>, rocket::Error>> {
    tokio::task::spawn(async { rocket.launch().await })
}

#[allow(unused)]
#[get("/auth/twitch/callback?<code>&<state>")]
async fn oauth_code_callback(
    code: String,
    state: String,
    auth_state: &State<RocketAuthState>,
) -> Result<String, LocalApiError> {
    let config = super::Config::new();
    let bot_name = config.twitch_bot_name.clone();
    if !auth_state.token_builder.csrf_is_valid(&state) {
        return Err(LocalApiError::StateMismatch { got: state });
    }

    #[derive(Serialize)]
    struct OauthPostBody {
        client_id: String,
        client_secret: String,
        code: String,
        grant_type: String,
        redirect_uri: String,
    }
    let body = OauthPostBody {
        client_id: config.twitch_client_id.clone(),
        client_secret: config.twitch_client_secret.clone(),
        code: code.clone(),
        grant_type: "authorization_code".to_string(),
        redirect_uri: config.twitch_redirect_url.clone(),
    };
    let client = reqwest::Client::new();
    #[cfg(not(test))]
    let response = client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&body)
        .send()
        .await
        .unwrap()
        .text()
        .await?;
    #[cfg(all(not(test), sensitive))]
    dbg!(&response);
    #[cfg(test)]
    let response = r#"{"access_token":"TestAccessToken", "refresh_token":"TestRefreshToken", "expires_in": 10, "scope": ["user_read"], "token_type": "bearer"}"#;

    let user_access_token =
        UserAccessToken::from(serde_json::from_str::<GetAccessTokenResponse>(&response)?);
    auth_state.auth_tx.send(user_access_token)?;

    Ok(format!("{bot_name} is authorized! you can close this tab"))
}

#[allow(unused)]
#[get("/?<error>&<error_description>", rank = 2)]
fn catch_oauth_error(error: String, error_description: String) -> String {
    let bot_name = super::Config::new().twitch_bot_name;
    eprintln!("caught an error with auth: {error}");
    eprintln!("{error_description}");

    match error.as_str() {
        "access_denied" => format!("{bot_name} was denied access to your account"),
        _ => format!("{bot_name} could not be authorized: {error_description} ({error})"),
    }
}

#[derive(Debug)]
enum LocalApiError {
    ParseError(String),
    RequestError(String),
    SendError(String),
    #[allow(unused)]
    StateMismatch {
        got: String,
    },
}

impl From<serde_json::Error> for LocalApiError {
    fn from(e: serde_json::Error) -> Self {
        Self::ParseError(format!("couldn't parse: {e}"))
    }
}

impl From<UserAccessToken> for LocalApiError {
    fn from(e: UserAccessToken) -> Self {
        Self::ParseError(format!("couldn't parse: {e:?}"))
    }
}

impl From<reqwest::Error> for LocalApiError {
    fn from(e: reqwest::Error) -> Self {
        Self::RequestError(format!("request failed: {e}"))
    }
}

impl From<SendError<UserAccessToken>> for LocalApiError {
    fn from(e: SendError<UserAccessToken>) -> Self {
        Self::SendError(format!("sending token failed: {e}"))
    }
}

impl Display for LocalApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalApiError::ParseError(e) => write!(f, "parsing failure! {e}"),
            LocalApiError::RequestError(e) => write!(f, "request burped with: {e}"),
            LocalApiError::SendError(e) => write!(f, "send error! {e}"),
            LocalApiError::StateMismatch { got } => write!(f, "state mismatch from twitch. be careful! this could mean someone is trying to do something malicious. (got code \"{got}\")"),
        }
    }
}

impl std::error::Error for LocalApiError {}

impl<'req> Responder<'req, 'static> for LocalApiError {
    fn respond_to(self, _request: &'req rocket::Request<'_>) -> rocket::response::Result<'static> {
        let display = format!("{self}");
        Response::build()
            .header(ContentType::Plain)
            .sized_body(display.len(), Cursor::new(display))
            .ok()
    }
}

#[allow(unused)]
struct RocketAuthState {
    auth_tx: Sender<UserAccessToken>,
    token_builder: UserTokenBuilder,
}

impl Clone for RocketAuthState {
    fn clone(&self) -> Self {
        let config = crate::CONFIG.clone();
        let client_id = config.twitch_client_id;
        let client_secret = config.twitch_client_secret;
        let redirect_url = config.twitch_redirect_url;
        Self {
            auth_tx: self.auth_tx.clone(),
            token_builder: UserTokenBuilder::new(
                client_id,
                client_secret,
                Url::parse(&redirect_url).unwrap(),
            )
            .set_scopes(SCOPE.to_vec()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CONFIG;
    use chrono::Utc;
    use twitch_oauth2::types::CsrfToken;

    use std::time::Duration;

    fn useraccesstoken() -> UserAccessToken {
        UserAccessToken {
            access_token: String::from("TestAccessToken"),
            refresh_token: String::from("TestRefreshToken"),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::from_secs(60_u64)),
        }
    }

    #[rocket::tokio::test]
    async fn it_works() {
        let config = CONFIG.clone();
        let client_id = config.twitch_client_id;
        let client_secret = config.twitch_client_secret;
        let redirect_url = config.twitch_redirect_url;

        let timeout = time::sleep(time::Duration::from_secs(1));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                biased;
                _ = &mut timeout => {
                    println!("api future timedout");
                    break;
                },
                res = new(client_id.clone(), client_secret.clone(), redirect_url.clone()) => {
                    if res.is_ok() {
                        let (token, handle) = res.unwrap();
                        println!("[Token] {token:?}");
                        handle.abort();
                        if handle.is_finished() || handle.await.unwrap_err().is_cancelled() {
                            println!("api handle has finished or was cancelled");
                            break;
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn oauth_error() {
        let mut error = String::from("access_denied");
        let error_description = String::from("Testing");
        let bot_name = crate::Config::new().twitch_bot_name;
        let res = catch_oauth_error(error, error_description.clone());
        assert_eq!(res, format!("{bot_name} was denied access to your account"));
        error = String::from("Bogus");
        let res2 = catch_oauth_error(error, error_description);
        assert_eq!(res2, format!("{bot_name} could not be authorized: Testing (Bogus)"));
    }

    #[tokio::test]
    async fn oauth_callback() {
        let code = String::from("TestC0DE");
        let csrf_token = CsrfToken::new_random_len(8_u32);
        let state = csrf_token.secret().to_string();
        let config = CONFIG.clone();
        let bot_name = config.twitch_bot_name;
        let client_id = config.twitch_client_id;
        let client_secret = config.twitch_client_secret;
        let redirect_url = config.twitch_redirect_url;
        let (auth_tx, _auth_rx) = mpsc::channel::<UserAccessToken>();
        let mut token_builder =
            UserTokenBuilder::new(client_id, client_secret, Url::parse(&redirect_url).unwrap())
                .set_scopes(SCOPE.to_vec());
        token_builder.set_csrf(csrf_token);
        let rocket = rocket::build().manage(RocketAuthState { auth_tx, token_builder });
        dbg!(&rocket);
        let auth_state = State::get(&rocket).expect("managed `RocketAuthState`");
        let expected = format!("{bot_name} is authorized! you can close this tab");
        match oauth_code_callback(code, state, auth_state).await {
            Ok(s) => assert_eq!(s, expected),
            Err(e) => panic!("{e:?}"),
        };
    }

    #[test]
    fn display_for_localapierror() {
        let _ = format!("{}", LocalApiError::ParseError(String::from("")));
        let _ = format!("{}", LocalApiError::RequestError(String::from("")));
        let _ = format!("{}", LocalApiError::SendError(String::from("")));
        let _ = format!("{}", LocalApiError::StateMismatch { got: String::from("") });
    }

    #[test]
    fn from_serdejsonerror_for_localapierror() {
        let broken = r#"{"access_token":"}"#;
        let e = serde_json::from_str::<UserAccessToken>(&broken).unwrap_err();
        let _: LocalApiError = From::from(e);
    }

    #[test]
    fn from_useracesstoken_for_localapierror() {
        let e = useraccesstoken();
        let _: LocalApiError = From::from(e);
    }

    #[rocket::tokio::test]
    async fn from_reqwesterror_for_localapierror() {
        let e = reqwest::get("bogus/url").await.unwrap_err();
        let _: LocalApiError = From::from(e);
    }

    #[test]
    fn from_senderror_for_localapierror() {
        let (tx, rx) = mpsc::channel::<UserAccessToken>();
        drop(rx);
        let res = tx.send(useraccesstoken());
        dbg!(&res);
        let e = res.unwrap_err();
        let _: LocalApiError = From::from(e);
    }
}
