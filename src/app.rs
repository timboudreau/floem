#[cfg(feature = "crossbeam")]
use crossbeam::channel::{Receiver};

#[cfg(not(feature = "crossbeam"))]
use std::sync::mpsc::{Receiver};

use crate::{app_events::UserEvent, application::spi::AppHandlerInternalAPI, AppConfig, AppEvent, WindowIdentifier};
use floem_reactive::{Runtime};

use crate::{
    application::app_handle::ApplicationHandle,
    view::IntoView,
    window::{WindowConfig, WindowCreation},
};

/// Initializes and runs an application with a single window.
///
/// This function creates a new `Application`, sets up a window with the provided view,
/// and starts the application event loop. The `app_view` closure is used to define
/// the root view of the application window.
///
/// Example:
/// ```no_run
/// floem::launch(|| "Hello, World!")
/// ```
///
/// To build an application and windows with more configuration, see [`Application`].
#[cfg_attr(debug_assertions, track_caller)]
pub fn launch<V: IntoView + 'static>(app_view: impl FnOnce() -> V + 'static) {
    Application::new().window(move |_| app_view(), None).run()
}

type EventLoopType = <ApplicationHandle as AppHandlerInternalAPI>::WindowingSystemEventLoop;

/// Floem top level application
/// This is the entry point of the application.
pub struct Application {
    pub(crate) receiver: Receiver<UserEvent>,
    pub(crate) handle: ApplicationHandle,
    pub(crate) initial_windows: Vec<WindowCreation>,

    #[cfg(all(feature = "winit", not(feature = "baseview")))]
    pub(crate) event_loop: Option<winit::event_loop::EventLoop>,
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub fn new() -> Self {
        Self::new_with_config(AppConfig::default())
    }

    pub fn on_event(mut self, action: impl Fn(AppEvent) + 'static) -> Self {
        self.handle.event_listener = Some(Box::new(action));
        self
    }

    /// Create a new window for the application, if you want multiple windows,
    /// just chain more window method to the builder.
    ///
    /// # Note
    ///
    /// Using `None` as a configuration argument is equivalent to using
    /// `WindowConfig::default()`.
    pub fn window<V: IntoView + 'static>(
        mut self,
        app_view: impl FnOnce(WindowIdentifier) -> V + 'static,
        config: Option<WindowConfig>,
    ) -> Self {
        self.initial_windows.push(WindowCreation {
            view_fn: Box::new(move |window_id: WindowIdentifier| app_view(window_id).into_any()),
            config,
        });
        self
    }

    /// Common pre-init tasks
    pub(crate) fn on_before_run() {
        Runtime::init_on_ui_thread();
        // Nudge UI when sync signals are updated from other threads.
        Runtime::set_sync_effect_waker(|| Application::send_proxy_event(UserEvent::Idle));
    }

    /// Run a function that handles inbound events or timer wakeups and takes the `ApplicationHandle`.
    /// The const-generic `USER_EVENTS` determines whether user events are also run - by using const
    /// generics for this, we ensure that two monomorphized versions of this method are emitted into
    /// the binary, and we don't pay a price at runtime.  Basically, the same set of calls to handle
    /// timers and call Runtime::drain_pending_work() surround more than one kind of event processing.
    ///
    /// This simply allows multiple windowing-system implementations to share this code without the risk
    /// of diverging due to having their own copies of it.
    #[inline(always)]
    pub(super) fn event_processing<F : FnOnce(&mut ApplicationHandle, &EventLoopType), const USER_EVENTS: bool>(&mut self, event_loop: &EventLoopType, f : F) {
        self.handle.handle_timer(event_loop);
        f(&mut self.handle, event_loop);
        if USER_EVENTS {
            for event in self.receiver.try_iter() {
                self.handle.handle_user_event(event_loop, event);
            }
        }
        if Runtime::has_pending_work() {
            Runtime::drain_pending_work();
        }
    }
}

/// Initiates the application shutdown process.
///
/// This function sends a `QuitApp` event to the application's event loop,
/// triggering the application to close gracefully.
pub fn quit_app() {
    Application::send_proxy_event(UserEvent::QuitApp);
}

/// Signals the application to reopen.
///
/// This function sends a `Reopen` event to the application's event loop.
/// It is safe to call from any thread.
pub fn reopen() {
    Application::send_proxy_event(UserEvent::Reopen {
        has_visible_windows: false,
    });
}
