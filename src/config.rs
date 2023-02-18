use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Error as IoError;

#[derive(Debug, Deserialize, Serialize)]
struct ConfigToml {
    #[cfg(feature = "discord")]
    discord: Option<ConfigTomlDiscord>,
    #[cfg(feature = "twitch")]
    twitch: Option<ConfigTomlTwitch>,
}

#[cfg(feature = "twitch")]
#[derive(Clone, Debug, Deserialize, Serialize)]
struct ConfigTomlTwitch {
    channels: Option<Vec<String>>,
}

#[cfg(feature = "discord")]
#[derive(Clone, Debug, Deserialize, Serialize)]
struct ConfigTomlDiscord {
    guildid: Option<String>,
    token: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[cfg(feature = "discord")]
    pub discord_guildid: String,
    #[cfg(feature = "discord")]
    pub discord_token: String,
    #[cfg(feature = "twitch")]
    pub twitch_channels: Vec<String>,
}

impl Config {
    pub fn new() -> Self {
        let config_filepaths: [&str; 2] = ["./config.toml", "./Cofig.toml"];
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
            eprintln!("Failed to create ConfigToml object out of config file.");
            ConfigToml {
                #[cfg(feature = "discord")]
                discord: None,
                #[cfg(feature = "twitch")]
                twitch: None,
            }
        });
        #[cfg(feature = "discord")]
        let discord_guildid: String = match config_toml.discord.clone() {
            Some(dgid) => dgid.guildid.unwrap_or_else(|| {
                eprintln!("Missing field `guildid` in table [discord]");
                "discord".to_string()
            }),
            None => {
                eprintln!("Missing table `[discord]`.");
                "discord".to_string()
            },
        };
        #[cfg(feature = "discord")]
        let discord_token: String = match config_toml.discord {
            Some(dt) => dt.token.unwrap_or_else(|| {
                eprintln!("Missing field `token` in table [discord]");
                "discord".to_string()
            }),
            None => {
                eprintln!("Missing table `[discord]`.");
                "discord".to_string()
            },
        };
        #[cfg(feature = "twitch")]
        let twitch_channels: Vec<String> = match config_toml.twitch {
            Some(twitch) => twitch.channels.unwrap_or_else(|| {
                eprintln!("Missing field `channels` in table [twitch]");
                vec!["twitch".to_owned()]
            }),
            None => {
                eprintln!("Missing table `[twitch]`.");
                vec!["twitch".to_owned()]
            },
        };
        Config {
            #[cfg(feature = "discord")]
            discord_guildid,
            #[cfg(feature = "discord")]
            discord_token,
            #[cfg(feature = "twitch")]
            twitch_channels,
        }
    }
}
