use ccg_bot_sys::nonstable;

#[nonstable(feature = "nonstable-testing")]
pub type NONSTABLETYPEWITHFEATURE = ();

#[nonstable(feature = "nonstable-testing")]
pub enum NONSTABLEENUMWITHFEATURE{}

#[nonstable(feature = "nonstable-testing")]
pub struct NonstableStructWithFeature{}

#[nonstable(feature = "nonstable-testing")]
pub fn nonstable_with_feature() {
 let _ = ();
}

#[nonstable(feature = "nonstable-testing")]
pub mod nonstable_mod_with_feature{}

#[nonstable(feature = "nonstable-testing")]
pub trait NONSTABLETRAITWITHFEATURE{}

#[nonstable(feature = "nonstable-testing")]
pub const NONSTABLE_CONST_WITH_FEATURE: i32 = 0_i32;

#[nonstable(feature = "nonstable-testing")]
pub static NONSTABLE_STATIC_WITH_FEATURE: i32 = 0_i32;

pub fn main() {
    let _ = nonstable_with_feature();
}