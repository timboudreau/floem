use crate::{WindowIdentifier, WindowUpdate, WindowingBackend, WindowingBackendInternal};

pub enum Baseview {}

impl WindowingBackend for Baseview {
    fn retreive_window_updates(id: &WindowIdentifier) -> Option<Vec<WindowUpdate>> {
        todo!()
    }
}

impl WindowingBackendInternal for Baseview {
    fn push_window_update(id: &WindowIdentifier, msg: WindowUpdate) {
        todo!()
    }
}
