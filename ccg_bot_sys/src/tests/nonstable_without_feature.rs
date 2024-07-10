use ccg_bot_sys::nonstable;

#[nonstable(issue = "1")]
pub type NONSTABLETYPEWITHOUTFEATURE = ();

#[nonstable(issue = "1")]
pub enum NONSTABLEENUMWITHOUTFEATURE{}

#[nonstable(issue = "1")]
pub struct NonstableStructWithoutFeature{}

#[nonstable(issue = "1")]
pub fn nonstable_without_feature() {
 let _ = ();
}

#[nonstable(issue = "1")]
pub mod nonstable_mod_without_feature{}

#[nonstable(issue = "1")]
pub trait NONSTABLETRAITWITHOUTFEATURE{}

#[nonstable(issue = "1")]
pub const NONSTABLE_CONST_WITHOUT_FEATURE: i32 = 0_i32;

#[nonstable(issue = "1")]
pub static NONSTABLE_STATIC_WITHOUT_FEATURE: i32 = 0_i32;

pub fn main() {
    let _ = nonstable_without_feature();
}