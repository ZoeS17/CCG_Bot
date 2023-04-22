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
        twitch_token: "".to_string(),
    }));
    let twitch_bool: bool = twitch.is_ok();
    assert!(twitch_bool);
}
