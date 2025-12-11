use windowing::internal_api::{WindowingBackend,WindowingSystem};
use crate::{
    ViewId,
    WindowIdentifier,
    id::RootViewProvider,
};

impl RootViewProvider for WindowIdentifier {
    fn root_view(&self) -> Option<ViewId> {
        WindowingSystem::root_view_in(self)
    }
}
