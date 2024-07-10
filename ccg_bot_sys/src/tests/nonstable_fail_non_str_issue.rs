use ccg_bot_sys::nonstable;

#[nonstable(issue = 1)]
static NONSTABLE_STATIC_WITH_NON_STR_ISSUE: i32 = 0_i32;

pub fn main() {
    let _ = NONSTABLE_STATIC_WITH_NON_STR_ISSUE;
}