use lazy_static::lazy_static;
use serenity::cache::Cache;
use serenity::model::user::CurrentUser;

lazy_static! {
    #[derive(Debug)]
    pub static ref CACHE: CurrentUser = Cache::default().current_user();
    pub static ref BOT_NAME: String = CACHE.clone().name;
    pub static ref BOT_URL: String = CACHE.clone().face();
}
