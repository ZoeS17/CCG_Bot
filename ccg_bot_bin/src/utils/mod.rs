#[cfg(any(feature = "discord", feature = "full"))]
pub mod commandinteraction;
pub mod json;
pub mod prelude {
    #[cfg(any(feature = "discord", feature = "full"))]
    pub use super::commandinteraction;
    pub use super::json::*;
}
