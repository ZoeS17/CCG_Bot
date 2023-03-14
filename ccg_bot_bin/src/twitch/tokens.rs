use crate::config::Config;
use twitch_irc::login::StaticLoginCredentials;

#[doc(hidden)]
pub(crate) fn load_token(config: Config) -> StaticLoginCredentials {
    let login_name = config.twitch_bot_name;
    let token = config.twitch_token;
    StaticLoginCredentials::new(login_name, Some(token))
}
