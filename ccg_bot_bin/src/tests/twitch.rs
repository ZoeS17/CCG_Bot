use crate::twitch::tokens::{BotTokenStorage, Token};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use twitch_irc::login::GetAccessTokenResponse;
use twitch_irc::login::TokenStorage;
use twitch_irc::login::UserAccessToken;
use twitch_oauth2::types::{AccessToken, ClientId, ClientSecret, RefreshToken};
use twitch_types::{UserId, UserName};

use std::time::{Duration, Instant};

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

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn it_works() {
    use super::super::config::Config;
    use super::super::twitch::new;
    let twitch = new(Config {
        database_url: "".to_string(),
        discord_guildid: "".to_string(),
        discord_token: "".to_string(),
        twitch_bot_name: "".to_string(),
        twitch_channels: vec!["".to_string()],
        twitch_client_id: "".to_string(),
        twitch_client_secret: "".to_string(),
        twitch_redirect_url: "http://localhost/".to_string(),
        bot_admins: vec!["test_admin".to_string()],
    })
    .await;
    let twitch_bool = twitch.is_ok();
    assert!(twitch_bool);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn token_from_useraccesstoken() {
    let now = Utc::now();
    let _uat: Token = From::from(UserAccessToken {
        access_token: String::from(""),
        refresh_token: String::from(""),
        created_at: now,
        // 300 seconds is 5 minutes
        expires_at: Some(now + chrono::Duration::seconds(300_i64)),
    });
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Instant_ {
    secs_since_epoch: u128,
    nanos_since_epoch: u128,
}

impl Instant_ {
    pub fn from_nanos(nanos: u64) -> Self {
        let secs_since_epoch = (nanos / 1_000_000_000_u64).into();
        let nanos_since_epoch = (nanos % 1_000_000_000_u64).into();
        Self { secs_since_epoch, nanos_since_epoch }
    }
}

#[test]
fn token_derives() {
    let interval = 300_usize;
    let expires_in = Duration::from_secs(interval as u64);
    let now = Utc::now();
    let struct_created = Instant::now();
    let later = now + chrono::Duration::seconds(interval as i64);
    let now_ts = now.timestamp();
    let later_ts = later.timestamp();
    let at = "test_access_token".to_string();
    let rt = "test_refresh_token".to_string();
    let clientid = ClientId::new(String::from("clientid"));
    let uid = UserId::new(String::from("12345")); // the same combination as your luggage?
    let name = UserName::new(String::from("username"));
    let client_secret = ClientSecret::new(String::from("clientsecret"));
    let token = Token {
        access_token: AccessToken::new(at),
        refresh_token: RefreshToken::new(rt),
        created_at: now,
        // 300 seconds is 5 minutes
        expires_at: later,
        struct_created,
        expires_in,
        clientid: clientid.clone(),
        uid: uid.clone(),
        name: name.clone(),
        client_secret: client_secret.clone(),
    };
    let _clone = token.clone();
    let debug = format!("{:?}", &token);
    dbg!(debug);
    let parsed = parse_instant_from_debug(struct_created);
    dbg!(&parsed);
    // we can unwrap here because it will cause a panic that will fail the test
    let _deserialize: Token = serde_json::from_value(serde_json::json!({
        "access_token": "test_access_token",
        "refresh_token": "test_refresh_token",
        "created_at": now_ts,
        "expires_at": later_ts,
        "struct_created": parsed,
        "expires_in": expires_in,
        "clientid": &clientid.take(),
        "uid": &uid.take(),
        "name": &name.take(),
        "client_secret": &client_secret.take(),
    }))
    .unwrap();
}

#[test]
fn bottokenstorage_derives() {
    let bot_token_storage = BotTokenStorage { prefix: "TWITCH".to_string() };
    let clone = bot_token_storage.clone();
    let _debug = format!("{:?}", &bot_token_storage);
    let serialize = serde_json::to_string(&bot_token_storage).unwrap();
    let _deserialize: BotTokenStorage = serde_json::from_str(&serialize).unwrap();
    //includes partial_eq
    let _eq = bot_token_storage.eq(&clone);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn token_from_getaccesstokenresponse() {
    let token_response = GetAccessTokenResponse {
        access_token: "TestAccessToken".to_string(),
        refresh_token: "TestRefreshToken".to_string(),
        expires_in: Some(300_u64),
    };
    let _token: Token = From::from(token_response);
}

#[test]
fn setup_load_and_update_tokens() {
    //setup
    let mut now = Utc::now();
    let mut expiry = Utc::now() + chrono::Duration::seconds(30_i64);
    let _ = std::env::set_var("TWITCH_ACCESS_TOKEN", "TestAccessTokenBefore");
    let _ = std::env::set_var("TWITCH_REFRESH_TOKEN", "TestRefreshTokenBefore");
    let _ = std::env::set_var("TWITCH_CREATED_AT", format!("{}", now));
    let _ = std::env::set_var("TWITCH_EXPIRES_AT", format!("{}", expiry));
    let bot_token_storage = BotTokenStorage { prefix: "TWITCH".to_string() };

    // before
    let bat = bot_token_storage.get_env::<String>("ACCESS_TOKEN").unwrap();
    let brt = bot_token_storage.get_env::<String>("REFRESH_TOKEN").unwrap();
    let bca = bot_token_storage.get_env::<DateTime<Utc>>("CREATED_AT").unwrap();
    let bea = bot_token_storage.get_env::<DateTime<Utc>>("EXPIRES_AT").unwrap();

    assert_eq!(&bat, "TestAccessTokenBefore");
    assert_eq!(&brt, "TestRefreshTokenBefore");
    assert_eq!(bca.to_string(), now.to_string());
    assert_eq!(bea.to_string(), expiry.to_string());

    // change

    now = Utc::now();
    expiry = Utc::now() + chrono::Duration::seconds(30_i64);

    let _ = bot_token_storage.set_env("ACCESS_TOKEN", "TestAccessTokenAfter");
    let _ = bot_token_storage.set_env("REFRESH_TOKEN", "TestRefreshTokenAfter");
    let _ = bot_token_storage.set_env("CREATED_AT", format!("{}", now));
    let _ = bot_token_storage.set_env("EXPIRES_AT", format!("{}", expiry));

    // after
    let aat = bot_token_storage.get_env::<String>("ACCESS_TOKEN").unwrap();
    let std_aat = std::env::var("TWITCH_ACCESS_TOKEN").unwrap();
    dbg!(std_aat);
    let art = bot_token_storage.get_env::<String>("REFRESH_TOKEN").unwrap();
    let std_art = std::env::var("TWITCH_REFRESH_TOKEN").unwrap();
    dbg!(std_art);
    let aca = bot_token_storage.get_env::<DateTime<Utc>>("CREATED_AT").unwrap();
    let std_aca = std::env::var("TWITCH_CREATED_AT").unwrap();
    dbg!(std_aca);
    let aea = bot_token_storage.get_env::<DateTime<Utc>>("EXPIRES_AT").unwrap();
    let std_aea = std::env::var("TWITCH_EXPIRES_AT").unwrap();
    dbg!(std_aea);

    assert_eq!(&aat, "TestAccessTokenAfter");
    assert_eq!(&art, "TestRefreshTokenAfter");
    assert_eq!(aca.to_string(), now.to_string());
    assert_eq!(aea.to_string(), expiry.to_string());

    assert_ne!(bat, aat);
    assert_ne!(brt, art);
    assert_ne!(bca, aca);
    assert_ne!(bea, aea);
}

#[tokio::test]
async fn bot_token_storage_load_token() -> Result<(), super::TwitchError> {
    let now = Utc::now();
    let expiry = now + chrono::Duration::seconds(7500_i64);
    let mut bot_token_storage = BotTokenStorage::new();
    let _ = std::env::set_var("TWITCH_ACCESS_TOKEN", "TestAccessToken");
    let _ = std::env::set_var("TWITCH_REFRESH_TOKEN", "TestRefreshToken");
    let _ = std::env::set_var("TWITCH_CREATED_AT", format!("{}", now));
    let _ = std::env::set_var("TWITCH_EXPIRES_AT", format!("{}", expiry));
    let loaded_token = bot_token_storage.load_token().await?;
    dbg!(&loaded_token);
    dbg!(&loaded_token.created_at.timestamp());
    dbg!(&now.timestamp());
    assert_eq!(loaded_token.access_token, "TestAccessToken".to_string());
    assert_eq!(loaded_token.refresh_token, "TestRefreshToken".to_string());
    // timestamp is needed because of the microsecond precision here. Seems unfortunate
    assert_eq!(loaded_token.created_at.timestamp(), now.timestamp());
    assert_eq!(loaded_token.expires_at.unwrap().timestamp(), expiry.timestamp());
    Ok(())
}

fn parse_instant_from_debug(instant: Instant /* , expected: Duration*/) -> Instant_ /*String */ {
    let mut vec: Vec<String> = format!("{instant:?}")
        .strip_prefix("Instant { tv_sec: ")
        .expect("Instant changed it's debug impl")
        .strip_suffix(" }")
        .expect("Instant changed it's debug impl")
        .split(',')
        .map(|i| i.to_string())
        .collect();
    vec[1] =
        vec[1].strip_prefix(" tv_nsec: ").expect("Instant changed it's debug impl").to_string();
    let secs = vec[0].parse::<i64>().expect("should never be negative");
    let nanos = vec[1].parse::<i32>().expect("literally can't be negative");
    let mul_nanos = (secs * 1_000_000_000_i64) + nanos as i64;
    // println!("{}", crate::utils::type_of(mul_nanos));
    Instant_::from_nanos(mul_nanos as u64)
}
