// Tests are grouped under this module so as to avoid
// having the test code itself included in coverage numbers.

#[allow(unused_imports)]
use super::*;

macro_rules! aw {
    ($e:expr) => {
        tokio_test::block_on($e)
    };
}

#[cfg(any(feature = "default", feature = "discord", feature = "full"))]
mod discord;
#[cfg(feature = "twitch")]
mod twitch;
