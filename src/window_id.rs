use windowing::{internal_api::WindowingBackend, ViewId, WindowIdentifier, WindowingSystem};
use crate::{
    id::RootViewProvider,
};

impl RootViewProvider for WindowIdentifier {
    fn root_view(&self) -> Option<ViewId> {
        WindowingSystem::root_view_in(self)
    }
}
