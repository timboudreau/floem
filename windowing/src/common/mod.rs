mod screen_layout;
mod view_id;
mod window_id_ext;

pub use screen_layout::*;
pub use view_id::*;
pub use window_id_ext::*;

/// Defines the type contract for an id that represents a window.
pub trait WindowIdDelegate:
    Copy + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash
{
}
