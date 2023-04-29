use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Error as IoError;

#[derive(Debug, Deserialize, Serialize)]
struct ConfigToml {
    #[cfg(any(feature = "discord", feature = "full"))]
    discord: Option<ConfigTomlDiscord>,
    #[cfg(any(feature = "twitch", feature = "full"))]
    twitch: Option<ConfigTomlTwitch>,
}

#[cfg(any(feature = "twitch", feature = "full"))]
#[derive(Clone, Debug, Deserialize, Serialize)]
struct ConfigTomlTwitch {
    channels: Option<Vec<String>>,
    token: Option<String>,
    bot_name: Option<String>,
}

#[cfg(any(feature = "discord", feature = "full"))]
#[derive(Clone, Debug, Deserialize, Serialize)]
struct ConfigTomlDiscord {
    guildid: Option<String>,
    token: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[cfg(any(feature = "discord", feature = "full"))]
    pub discord_guildid: String,
    #[cfg(any(feature = "discord", feature = "full"))]
    pub discord_token: String,
    #[cfg(any(feature = "twitch", feature = "full"))]
    pub twitch_channels: Vec<String>,
    #[cfg(any(feature = "twitch", feature = "full"))]
    pub twitch_token: String,
    #[cfg(any(feature = "twitch", feature = "full"))]
    pub twitch_bot_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(any(feature = "discord", feature = "full"))]
            discord_guildid: "0".to_string(),
            #[cfg(any(feature = "discord", feature = "full"))]
            discord_token: Default::default(),
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch_channels: Default::default(),
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch_token: Default::default(),
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch_bot_name: Default::default(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        #[cfg(not(test))]
        let config_filepaths: [&str; 2] = ["./config.toml", "./Config.toml"];
        #[cfg(test)]
        let config_filepaths: [&str; 1] = ["./config.toml.example"];

        let mut content: String = "".to_owned();
        for filepath in config_filepaths {
            let result: Result<String, IoError> = fs::read_to_string(filepath);
            //not sure why clippy hates this
            #[allow(clippy::unnecessary_unwrap)]
            if result.is_ok() {
                content = result.unwrap();
                break;
            }
        }
        // All this because it isn't read otherwise... The heck Rust
        #[cfg(not(any(feature = "discord", feature = "full", feature = "twitch")))]
        let _: ConfigToml = toml::from_str(&content).unwrap_or_else(|_| ConfigToml {});
        #[cfg(any(feature = "twitch", feature = "discord", feature = "full"))]
        let config_toml: ConfigToml = toml::from_str(&content).unwrap_or_else(|_| {
            error!("Failed to create ConfigToml object out of config file.");
            ConfigToml {
                #[cfg(any(feature = "discord", feature = "full"))]
                discord: None,
                #[cfg(any(feature = "twitch", feature = "full"))]
                twitch: None,
            }
        });
        #[cfg(any(feature = "discord", feature = "full"))]
        let discord_guildid: String = match config_toml.discord.clone() {
            Some(dgid) => dgid.guildid.unwrap_or_else(|| {
                error!("Missing field `guildid` in table [discord]");
                "0".to_string()
            }),
            None => {
                eprintln!("Missing table `[discord]`.");
                "0".to_string()
            },
        };
        #[cfg(any(feature = "discord", feature = "full"))]
        let discord_token: String = match config_toml.discord {
            Some(dt) => dt.token.unwrap_or_else(|| {
                error!("Missing field `token` in table [discord]");
                "discord".to_string()
            }),
            None => {
                error!("Missing table `[discord]`.");
                "discord".to_string()
            },
        };
        #[cfg(any(feature = "twitch", feature = "full"))]
        let twitch_bot_name: String = match config_toml.twitch.clone() {
            Some(tbn) => tbn.bot_name.unwrap_or_else(|| {
                error!("Missing field `bot_name` in table [twitch]");
                "twitch".to_string()
            }),
            None => {
                error!("Missing table `[twitch]`.");
                "twitch".to_string()
            },
        };
        #[cfg(any(feature = "twitch", feature = "full"))]
        let twitch_channels: Vec<String> = match config_toml.twitch.clone() {
            Some(tc) => tc.channels.unwrap_or_else(|| {
                error!("Missing field `channels` in table [twitch]");
                vec!["twitch".to_owned()]
            }),
            None => {
                error!("Missing table `[twitch]`.");
                vec!["twitch".to_owned()]
            },
        };
        #[cfg(any(feature = "twitch", feature = "full"))]
        let twitch_token: String = match config_toml.twitch {
            Some(tt) => tt.token.unwrap_or_else(|| {
                error!("Missing field `token` in table [twitch]");
                "twitch".to_string()
            }),
            None => {
                error!("Missing table `[twitch]`.");
                "twitch".to_string()
            },
        };
        Config {
            #[cfg(any(feature = "discord", feature = "full"))]
            discord_guildid,
            #[cfg(any(feature = "discord", feature = "full"))]
            discord_token,
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch_channels,
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch_token,
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch_bot_name,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::json::prelude::{from_str, to_string};

    #[cfg(any(feature = "discord", feature = "full"))]
    #[test]
    fn derives_config_toml_discord() {
        let all_some = ConfigTomlDiscord {
            guildid: Some("12345678910111213".to_string()),
            token: Some("AbcDEFGhJkl0MnO1PQRsTUvx.Abcdef.AbCDefgHiJkLMNOpqrSTU0vWXy1".to_string()),
        };
        let _guild_id_some =
            ConfigTomlDiscord { guildid: Some("12345678910111213".to_string()), token: None };
        let _token_some = ConfigTomlDiscord {
            guildid: None,
            token: Some("AbcDEFGhJkl0MnO1PQRsTUvx.Abcdef.AbCDefgHiJkLMNOpqrSTU0vWXy1".to_string()),
        };
        let _all_none = ConfigTomlDiscord { guildid: None, token: None };
        let all_some_string = to_string(&all_some).unwrap(); // derive(Serialize)
        let _: ConfigTomlDiscord = from_str(&all_some_string).unwrap(); // derive(Deserialize)
        let _ = all_some.clone(); // derive(Clone)
        let _ = format!("{:?}", all_some); // derive(Debug)
    }

    #[cfg(any(feature = "twitch", feature = "full"))]
    #[test]
    fn derives_config_toml_twitch() {
        let all_some = ConfigTomlTwitch {
            channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
            token: Some("".to_string()),
            bot_name: Some("".to_string()),
        };
        let _channels_some = ConfigTomlTwitch {
            channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
            token: None,
            bot_name: None,
        };
        let _token_some =
            ConfigTomlTwitch { channels: None, token: Some("".to_string()), bot_name: None };
        let _bot_name_some =
            ConfigTomlTwitch { channels: None, token: None, bot_name: Some("".to_string()) };
        let _first_pair_some = ConfigTomlTwitch {
            channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
            token: Some("".to_string()),
            bot_name: None,
        };
        let _second_pair_some = ConfigTomlTwitch {
            channels: None,
            token: Some("".to_string()),
            bot_name: Some("".to_string()),
        };
        let _outer_some = ConfigTomlTwitch {
            channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
            token: None,
            bot_name: Some("".to_string()),
        };
        let _all_none = ConfigTomlTwitch { channels: None, token: None, bot_name: None };
        let _first_pair_none =
            ConfigTomlTwitch { channels: None, token: None, bot_name: Some("".to_string()) };
        let _second_pair_none = ConfigTomlTwitch {
            channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
            token: None,
            bot_name: None,
        };
        let _outer_none =
            ConfigTomlTwitch { channels: None, token: Some("".to_string()), bot_name: None };
        let all_some_string = to_string(&all_some).unwrap(); // derive(Serialize)
        let _: ConfigTomlTwitch = from_str(&all_some_string).unwrap(); // derive(Deserialize)
        let _ = all_some.clone(); // derive(Clone)
        let _ = format!("{:?}", all_some); // derive(Debug)
    }

    #[test]
    fn derives_config_toml() {
        let all_some = ConfigToml {
            #[cfg(any(feature = "discord", feature = "full"))]
            discord: Some(ConfigTomlDiscord {
                guildid: Some("".to_string()),
                token: Some("".to_string()),
            }),
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch: Some(ConfigTomlTwitch {
                channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
                token: Some("".to_string()),
                bot_name: Some("".to_string()),
            }),
        };
        let _discord_some = ConfigToml {
            #[cfg(any(feature = "discord", feature = "full"))]
            discord: Some(ConfigTomlDiscord {
                guildid: Some("".to_string()),
                token: Some("".to_string()),
            }),
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch: None,
        };
        let _twitch_some = ConfigToml {
            #[cfg(any(feature = "discord", feature = "full"))]
            discord: None,
            #[cfg(any(feature = "twitch", feature = "full"))]
            twitch: Some(ConfigTomlTwitch {
                channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
                token: Some("".to_string()),
                bot_name: Some("".to_string()),
            }),
        };
        let all_some_string = to_string(&all_some).unwrap(); // derive(Serialize)
        let _: ConfigToml = from_str(&all_some_string).unwrap(); // derive(Deserialize)
        let _ = format!("{:?}", all_some); // derive(Debug)
    }

    #[test]
    fn impl_config_new() {
        let _ = Config::new();
    }
}
