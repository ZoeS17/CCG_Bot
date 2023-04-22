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
        let config_filepaths: [&str; 2] = ["./config.toml", "./Config.toml"];
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
