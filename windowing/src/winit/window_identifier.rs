use std::ops::{Deref, DerefMut};
use winit::window::WindowId;

/// A transparent wrapper over the library handling window management's window
/// identity abstraction.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct WindowIdentifier {
    id: WindowId,
}

impl crate::common::WindowIdDelegate for WindowIdentifier {}

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
        Self::new(id)
    }
}

impl From<WindowIdentifier> for WindowId {
    fn from(value: WindowIdentifier) -> Self {
        value.into_inner()
    }
}

impl From<&WindowId> for WindowIdentifier {
    fn from(id: &WindowId) -> Self {
        Self::new(id.to_owned())
    }
}

impl From<&WindowIdentifier> for WindowId {
    fn from(value: &WindowIdentifier) -> Self {
        value.id.to_owned()
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
