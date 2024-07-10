use ccg_bot_sys::nonstable;

#[nonstable()]
pub type NONSTABLETYPEWITHOUTFEATUREORISSUE = ();

#[nonstable()]
pub enum NONSTABLEENUMWITHOUTFEATUREORISSUE{}

#[nonstable()]
pub struct NonstableStructWithoutFeatureOrIssue{}

#[nonstable()]
pub fn nonstable_without_feature_or_issue() {
 let _ = ();
}

#[nonstable()]
pub mod nonstable_mod_without_feature_or_issue{}

#[nonstable()]
pub trait NONSTABLETRAITWITHOUTFEATUREORISSUE{}

#[nonstable()]
pub const NONSTABLE_CONST_WITHOUT_FEATURE_OR_ISSUE: i32 = 0_i32;

#[nonstable()]
pub static NONSTABLE_STATIC_WITHOUT_FEATURE_OR_ISSUE: i32 = 0_i32;

pub fn main() {
    let _ = nonstable_without_feature_or_issue();
}