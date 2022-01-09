//! Simple data types and enums
mod geometry;
mod window;

pub use geometry::*;
pub use window::*;

/// Increment / decrement a value
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Change {
    /// increase the value
    More,
    /// decrease the value, possibly clamping
    Less,
}
