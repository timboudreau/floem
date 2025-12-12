pub use winit::icon::{Icon, RgbaIcon};
pub use winit::monitor::Fullscreen;
pub use winit::window::Theme;
pub use winit::window::WindowButtons;
pub use winit::window::WindowId;
pub use winit::window::WindowLevel;

use crate::AnyView;
use crate::WindowIdentifier;
use crate::app_events::{add_app_update_event, AppUpdateEvent};
use crate::view::IntoView;

// For backward compatibility, these need to be namespaced into `crate::window`:
pub use crate::config::window_config::*;
pub use crate::config::window_config_mac::*;
pub use crate::config::window_config_web::*;
pub use crate::config::window_config_win::*;

pub struct WindowCreation {
    pub(crate) view_fn: Box<dyn FnOnce(WindowIdentifier) -> AnyView>,
    pub(crate) config: Option<WindowConfig>,
}

/// Create a new window. You'll need to create Application first, otherwise it
/// will panic.
pub fn new_window<V: IntoView + 'static>(
    app_view: impl FnOnce(WindowIdentifier) -> V + 'static,
    config: Option<WindowConfig>,
) {
    add_app_update_event(AppUpdateEvent::NewWindow {
        window_creation: WindowCreation {
            view_fn: Box::new(|window_id| app_view(window_id.into()).into_any()),
            config,
        },
    });
}

/// request the window to be closed
pub fn close_window(window_id: WindowIdentifier) {
    add_app_update_event(AppUpdateEvent::CloseWindow { window_id });
}
