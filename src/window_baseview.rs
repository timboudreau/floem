use crate::AnyView;
use crate::WindowIdentifier;
use crate::app::{AppUpdateEvent, add_app_update_event};
use crate::view::IntoView;
use peniko::kurbo::Size;

pub struct WindowCreation {
    pub(crate) view_fn: Box<dyn FnOnce(WindowId) -> AnyView>,
    pub(crate) config: Option<WindowConfig>,
}

/// Baseview has very few options for window configuration, since the window is
/// provided by a host application that has already created it.
pub struct WindowConfig {
    pub(crate) size: Option<Size>,
}

impl WindowConfig {
    /// Requests the window to be of specific dimensions.
    ///
    /// If this is not set, some platform-specific dimensions will be used.
    #[inline]
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = Some(size.into());
        self
    }
}

/// Create a new window. You'll need to create Application first, otherwise it
/// will panic.
pub fn new_window<V: IntoView + 'static>(
    app_view: impl FnOnce(WindowId) -> V + 'static,
    config: Option<WindowConfig>,
) {
    add_app_update_event(AppUpdateEvent::NewWindow {
        window_creation: WindowCreation {
            view_fn: Box::new(|window_id| app_view(window_id).into_any()),
            config,
        },
    });
}

/// request the window to be closed
pub fn close_window(window_id: WindowId) {
    add_app_update_event(AppUpdateEvent::CloseWindow { window_id });
}
