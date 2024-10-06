use crate::env;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Error as IoError;

#[derive(Debug, Deserialize, Serialize)]
struct ConfigToml {
    database: Option<ConfigTomlDatabase>,
    discord: Option<ConfigTomlDiscord>,
    twitch: Option<ConfigTomlTwitch>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ConfigTomlTwitch {
    channels: Option<Vec<String>>,
    client_id: Option<String>,
    client_secret: Option<String>,
    bot_name: Option<String>,
    redirect_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ConfigTomlDiscord {
    guildid: Option<String>,
    token: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ConfigTomlDatabase {
    database_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub database_url: String,
    pub discord_guildid: String,
    pub discord_token: String,
    pub twitch_channels: Vec<String>,
    pub twitch_client_id: String,
    pub twitch_client_secret: String,
    pub twitch_bot_name: String,
    pub twitch_redirect_url: String,
    pub bot_admins: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: Default::default(),
            discord_guildid: "0".to_string(),
            discord_token: Default::default(),
            twitch_channels: Default::default(),
            twitch_client_id: Default::default(),
            twitch_client_secret: Default::default(),
            twitch_bot_name: Default::default(),
            twitch_redirect_url: Default::default(),
            bot_admins: Default::default(),
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
        let config_toml_result: Result<ConfigToml, toml::de::Error> = toml::from_str(&content);
        let config_toml: ConfigToml = config_toml_result.unwrap_or_else(|_| {
            eprintln!("Failed to create ConfigToml object out of config file.");
            ConfigToml { database: None, discord: None, twitch: None }
        });
        let database_url: String = match config_toml.database.clone() {
            Some(db) => db.database_url.unwrap_or_else(|| {
                eprintln!("Missing field `databaseurl` in table [database]");
                env::var("DATABASE_URL").unwrap_or_default()
            }),
            None => {
                eprintln!("Missing table `[database]`.");
                Default::default()
            },
        };
        let discord_guildid: String = match config_toml.discord.clone() {
            Some(dgid) => {
                #[cfg(test)]
                let _ = dgid;
                #[cfg(test)]
                let di = env::var("DISCORD_GUILD_ID").unwrap_or_else(|_| "0".to_string());
                #[cfg(not(test))]
                let di = dgid.guildid.unwrap_or_else(|| {
                    eprintln!("Missing field `guildid` in table [discord]");
                    env::var("DISCORD_GUILD_ID").unwrap_or_else(|_| "0".to_string())
                });
                di
            },
            None => {
                eprintln!("Missing table `[discord]`.");
                "0".to_string()
            },
        };
        let discord_token: String = match config_toml.discord {
            Some(dt) => {
                #[cfg(test)]
                let token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| "discord".to_string());
                #[cfg(test)]
                let _ = dt;
                #[cfg(not(test))]
                let token = dt.token.unwrap_or_else(|| {
                    eprintln!("Missing field `token` in table [discord]");
                    env::var("DISCORD_TOKEN").unwrap_or_else(|_| "discord".to_string())
                });
                token
            },
            None => {
                eprintln!("Missing table `[discord]`.");
                "discord".to_string()
            },
        };
        let twitch_bot_name: String = match config_toml.twitch.clone() {
            Some(tbn) => tbn.bot_name.unwrap_or_else(|| {
                eprintln!("Missing field `bot_name` in table [twitch]");
                "twitch".to_string()
            }),
            None => {
                eprintln!("Missing table `[twitch]`.");
                "twitch".to_string()
            },
        };
        let twitch_channels: Vec<String> = match config_toml.twitch.clone() {
            Some(tc) => tc.channels.unwrap_or_else(|| {
                eprintln!("Missing field `channels` in table [twitch]");
                vec!["twitch".to_owned()]
            }),
            None => {
                eprintln!("Missing table `[twitch]`.");
                vec!["twitch".to_owned()]
            },
        };
        let twitch_client_id: String = match config_toml.twitch.clone() {
            Some(tci) => tci.client_id.unwrap_or_else(|| {
                eprintln!("Missing field `client_id` in table [twitch]");
                "twitch".to_string()
            }),
            None => {
                eprintln!("Missing table `[twitch]`.");
                "twitch".to_string()
            },
        };
        let twitch_client_secret: String = match config_toml.twitch.clone() {
            Some(tcs) => tcs.client_secret.unwrap_or_else(|| {
                eprintln!("Missing field `client_secret` in table [twitch]");
                "twitch".to_string()
            }),
            None => {
                eprintln!("Missing table `[twitch]`.");
                "twitch".to_string()
            },
        };
        let twitch_redirect_url: String = match config_toml.twitch {
            Some(rdu) => rdu.redirect_url.unwrap_or_else(|| {
                eprintln!("Missing field `redirect_url` in table [twitch]");
                "twitch".to_string()
            }),
            None => {
                eprintln!("Missing table `[twitch]`.");
                "twitch".to_string()
            },
        };
        let bot_admins = fs::read_to_string("./admins")
            .unwrap_or_default()
            .trim()
            .split('\n')
            .map(|i| i.to_string())
            .collect();
        Config {
            database_url,
            discord_guildid,
            discord_token,
            twitch_channels,
            twitch_client_id,
            twitch_client_secret,
            twitch_bot_name,
            twitch_redirect_url,
            bot_admins,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::json::{from_str, to_string};

    #[test]
    fn derives_config_toml_database() {
        let all_some = ConfigTomlDatabase { database_url: Some(Default::default()) };

        let _url_none = ConfigTomlDatabase { database_url: None };
        let all_some_string = to_string(&all_some).unwrap(); // derive(Serialize)
        let _: ConfigTomlDatabase = from_str(&all_some_string).unwrap(); // derive(Deserialize)
        let _ = all_some.clone(); // derive(Clone)
        let _ = format!("{:?}", all_some); // derive(Debug)
    }

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

    #[test]
    fn derives_config_toml_twitch() {
        let all_some = ConfigTomlTwitch {
            channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
            client_id: Some("".to_string()),
            client_secret: Some("".to_string()),
            bot_name: Some("".to_string()),
            redirect_url: Some("".to_string()),
        };
        let _channels_some = ConfigTomlTwitch {
            channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
            client_id: None,
            client_secret: None,
            bot_name: None,
            redirect_url: None,
        };
        let _client_id_some = ConfigTomlTwitch {
            channels: None,
            client_id: Some("".to_string()),
            client_secret: None,
            bot_name: None,
            redirect_url: None,
        };
        let _client_secret_some = ConfigTomlTwitch {
            channels: None,
            client_id: None,
            client_secret: Some("".to_string()),
            bot_name: None,
            redirect_url: None,
        };
        let _bot_name_some = ConfigTomlTwitch {
            channels: None,
            client_id: None,
            client_secret: None,
            bot_name: Some("".to_string()),
            redirect_url: None,
        };
        let _first_pair_some = ConfigTomlTwitch {
            channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
            client_id: Some("".to_string()),
            client_secret: None,
            bot_name: None,
            redirect_url: None,
        };
        let _second_pair_some = ConfigTomlTwitch {
            channels: None,
            client_id: Some("".to_string()),
            client_secret: Some("".to_string()),
            bot_name: None,
            redirect_url: None,
        };
        let _one_three_some = ConfigTomlTwitch {
            channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
            client_id: None,
            client_secret: Some("".to_string()),
            bot_name: None,
            redirect_url: None,
        };
        let _two_four_some = ConfigTomlTwitch {
            channels: None,
            client_id: Some("".to_string()),
            client_secret: None,
            bot_name: Some("".to_string()),
            redirect_url: None,
        };
        let _outer_some = ConfigTomlTwitch {
            channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
            client_id: None,
            client_secret: None,
            bot_name: Some("".to_string()),
            redirect_url: None,
        };
        let _all_none = ConfigTomlTwitch {
            channels: None,
            client_id: None,
            client_secret: None,
            bot_name: None,
            redirect_url: None,
        };
        let _first_pair_none = ConfigTomlTwitch {
            channels: None,
            client_id: None,
            client_secret: Some("".to_string()),
            bot_name: Some("".to_string()),
            redirect_url: None,
        };
        let _second_pair_none = ConfigTomlTwitch {
            channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
            client_id: None,
            client_secret: None,
            bot_name: Some("".to_string()),
            redirect_url: None,
        };
        let _outer_none = ConfigTomlTwitch {
            channels: None,
            client_id: Some("".to_string()),
            client_secret: Some("".to_string()),
            bot_name: None,
            redirect_url: None,
        };
        let all_some_string = to_string(&all_some).unwrap(); // derive(Serialize)
        let _: ConfigTomlTwitch = from_str(&all_some_string).unwrap(); // derive(Deserialize)
        let _ = all_some.clone(); // derive(Clone)
        let _ = format!("{:?}", all_some); // derive(Debug)
    }

    #[test]
    fn derives_config_toml() {
        let all_some = ConfigToml {
            database: Some(ConfigTomlDatabase { database_url: Some("".to_string()) }),
            discord: Some(ConfigTomlDiscord {
                guildid: Some("".to_string()),
                token: Some("".to_string()),
            }),
            twitch: Some(ConfigTomlTwitch {
                channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
                client_id: Some("".to_string()),
                client_secret: Some("".to_string()),
                bot_name: Some("".to_string()),
                redirect_url: Some("".to_string()),
            }),
        };
        let _discord_some = ConfigToml {
            database: None,
            discord: Some(ConfigTomlDiscord {
                guildid: Some("".to_string()),
                token: Some("".to_string()),
            }),
            twitch: None,
        };
        let _twitch_some = ConfigToml {
            database: None,
            discord: None,
            twitch: Some(ConfigTomlTwitch {
                channels: Some(vec!["Twitch".to_string(), "TwitchRivals".to_string()]),
                client_id: Some("".to_string()),
                client_secret: Some("".to_string()),
                bot_name: Some("".to_string()),
                redirect_url: Some("".to_string()),
            }),
        };
        let _database_some = ConfigToml {
            database: Some(ConfigTomlDatabase { database_url: Some("".to_string()) }),
            discord: None,
            twitch: None,
        };
        let all_some_string = to_string(&all_some).unwrap(); // derive(Serialize)
        let _: ConfigTomlTwitch = from_str(&all_some_string).unwrap(); // derive(Deserialize)
        let _ = format!("{:?}", all_some); // derive(Debug)
    }

    #[test]
    fn impl_config_new() {
        let _ = Config::new();
    }
}
