use crate::{
    internal_api::{WindowUpdate, WindowingBackend, WindowingBackendInternal},
};
use peniko::kurbo::Size;
use super::window_identifier::WindowIdentifier;

pub enum Baseview {}

impl WindowingBackend for Baseview {
    fn retreive_window_updates(id: &WindowIdentifier) -> Option<Vec<WindowUpdate>> {
        todo!()
    }

    /// Called by `ApplicationHandle` at the end of the event loop callback to process window updates.
    fn process_window_updates(id: &WindowIdentifier) -> bool {
        todo!()
    }

    fn logical_surface_size(window: &super::NativeWindowInner, scale: f64) -> Size {
        todo!()
    }
}

impl WindowingBackendInternal for Baseview {
    fn push_window_update(id: &WindowIdentifier, msg: WindowUpdate) {
        todo!()
    }
}
