use ccg_bot_sys::nonstable;

#[nonstable(feature = "nonstable-testing", issue = "1")]
pub type NONSTABLETYPEWITHFEATURE = ();

#[nonstable(feature = "nonstable-testing", issue = "1")]
pub enum NONSTABLEENUMWITHFEATURE{}

#[nonstable(feature = "nonstable-testing", issue = "1")]
pub struct NonstableStructWithFeature{}

#[nonstable(feature = "nonstable-testing", issue = "1")]
pub fn nonstable_with_feature() {
 let _ = ();
}

#[nonstable(feature = "nonstable-testing", issue = "1")]
pub mod nonstable_mod_with_feature{}

#[nonstable(feature = "nonstable-testing", issue = "1")]
pub trait NONSTABLETRAITWITHFEATURE{}

#[nonstable(feature = "nonstable-testing", issue = "1")]
pub const NONSTABLE_CONST_WITH_FEATURE: i32 = 0_i32;

#[nonstable(feature = "nonstable-testing", issue = "1")]
pub static NONSTABLE_STATIC_WITH_FEATURE: i32 = 0_i32;

pub fn main() {
    let _ = nonstable_with_feature();
}