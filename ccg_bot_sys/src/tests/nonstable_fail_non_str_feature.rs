use ccg_bot_sys::nonstable;

#[nonstable(feature = 1)]
static NONSTABLE_STATIC_WITH_NON_STR_FEATURE: i32 = 0_i32;

pub fn main() {
    let _ = NONSTABLE_STATIC_WITH_NON_STR_FEATURE;
}