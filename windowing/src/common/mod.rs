mod screen_layout;

pub use screen_layout::*;

/// Defines the type contract for an id that represents a window.
pub trait WindowIdDelegate: Copy + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash{}
