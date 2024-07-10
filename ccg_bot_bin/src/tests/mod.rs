// Tests are grouped under this module so as to avoid
// having the test code itself included in coverage numbers.

#[allow(unused_imports)]
use super::*;

// macro_rules! aw {
//     ($e:expr) => {
//         tokio_test::block_on($e)
//     };
// }

pub mod twitch;

#[test]
fn clippy_dbg() {
    dbg!("test clippy::dbg_macro lint regression");
}
