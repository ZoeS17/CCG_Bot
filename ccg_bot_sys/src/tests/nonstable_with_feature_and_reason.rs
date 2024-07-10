use ccg_bot_sys::nonstable;

#[nonstable(feature = "nonstable-testing", reason = "Modifying the phase variance")]
pub type NONSTABLETYPEWITHFEATUREANDREASON = ();

#[nonstable(feature = "nonstable-testing", reason = "Modifying the phase variance")]
pub enum NONSTABLEENUMWITHFEATUREANDREASON{}

#[nonstable(feature = "nonstable-testing", reason = "Modifying the phase variance")]
pub struct NonstableStructWithFeatureAndReason{}

#[nonstable(feature = "nonstable-testing", reason = "Modifying the phase variance")]
pub fn nonstable_with_feature_and_reason() {
 let _ = ();
}

#[nonstable(feature = "nonstable-testing", reason = "Modifying the phase variance")]
pub mod nonstable_mod_with_feature_and_reason{}

#[nonstable(feature = "nonstable-testing", reason = "Modifying the phase variance")]
pub trait NONSTABLETRAITWITHFEATUREANDREASON{}

#[nonstable(feature = "nonstable-testing", reason = "Modifying the phase variance")]
pub const NONSTABLE_CONST_WITH_FEATURE_AND_REASON: i32 = 0_i32;

#[nonstable(feature = "nonstable-testing", reason = "Modifying the phase variance")]
pub static NONSTABLE_STATIC_WITH_FEATURE_AND_REASON: i32 = 0_i32;

pub fn main() {
    let _ = nonstable_with_feature_and_reason();
}