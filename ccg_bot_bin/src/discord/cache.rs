use lazy_static::lazy_static;
use serenity::cache::Cache;
use serenity::model::user::CurrentUser;

lazy_static! {
    #[derive(Debug, Eq, PartialEq)]
    pub static ref CACHE: CurrentUser = Cache::default().current_user();
    #[derive(Debug, Eq, PartialEq)]
    pub static ref BOT_NAME: String = CACHE.clone().name;
    #[derive(Debug, Eq, PartialEq)]
    pub static ref BOT_URL: String = CACHE.clone().face();
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn cache_debug() {
        let out = format!("{:?}", CACHE);
        assert_eq!(out, "CACHE { __private_field: () }")
    }
    #[test]
    fn bot_name_static() {
        let out = format!("{:?}", BOT_NAME);
        assert_eq!(out, "BOT_NAME { __private_field: () }")
    }
    #[test]
    fn bot_url_static() {
        let out = format!("{:?}", BOT_URL);
        assert_eq!(out, "BOT_URL { __private_field: () }")
    }
}
