use std::ops::{Deref, DerefMut};
use winit::window::WindowId;

/// Defines the type contract for an id that represents a window.
pub trait WindowIdDelegate:
    Copy + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash
{
}

/// A transparent wrapper over the library handling window management's window
/// identity abstraction.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct WindowIdentifier {
    id: WindowId,
}

impl WindowIdDelegate for WindowIdentifier {}

impl WindowIdentifier {
    pub const fn new(id: WindowId) -> Self {
        Self { id }
    }

    pub const fn into_inner(self) -> WindowId {
        self.id
    }
}

impl From<WindowId> for WindowIdentifier {
    fn from(id: WindowId) -> Self {
        Self { id }
    }
}

impl From<WindowIdentifier> for WindowId {
    fn from(value: WindowIdentifier) -> Self {
        value.into_inner()
    }
}

impl Deref for WindowIdentifier {
    type Target = WindowId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl DerefMut for WindowIdentifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.id
    }
}
