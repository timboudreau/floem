use crate::{app_events::UserEvent, application::{
        app_handle::ApplicationHandle, baseview_hacks::BaseviewPseudoEventLoop, spi::AppHandlerInternalAPI
    }, window::{WindowConfig, WindowCreation}, AppConfig, AppEvent, IntoView
};
use super::app::*;
use baseview::WindowHandler;
use floem_reactive::Runtime;
use windowing::public_api::WindowIdentifier;

#[cfg(feature = "crossbeam")]
use crossbeam::channel::{unbounded as channel};

#[cfg(not(feature = "crossbeam"))]
use std::sync::mpsc::{Receiver, Sender, channel};

#[cfg(feature = "crossbeam")]
use crossbeam::channel::{Receiver, Sender};

use std::{cell::RefCell, sync::Arc};

type EventLoopType = <ApplicationHandle as AppHandlerInternalAPI>::WindowingSystemEventLoop;

/*
For baseview, we need to create listeners over an Application which can obtain a mutable reference
to it, and they need to know which window id they reference.  So, we bury the actual implementation
in an Arc<RefCell>.
*/

pub struct Application {
    pub(super) inner : Arc<RefCell<ApplicationInner>>,
}

impl Application {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn run(mut self) {
        Self::on_before_run();
        todo!()
    }

    pub(crate) fn send_proxy_event(event: UserEvent) {
        todo!()
    }

    pub fn new_with_config(config: AppConfig) -> Self {
        crate::screen_layout::ensure_windowing_system_initialized();

        #[cfg(target_os = "macos")]
        crate::app_delegate::set_app_delegate();

        let (sender, receiver) = channel();

        // Pending - any baseview clipboard init
        let handle = ApplicationHandle::new(config);

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
            use crate::{app_events::{add_app_update_event, AppUpdateEvent}};

            add_app_update_event(AppUpdateEvent::MenuAction {
                action_id: event.id,
            });
        }));
        Self {
            inner : Arc::new(RefCell::new(ApplicationInner {
                receiver: receiver,
                handle: handle,
                initial_windows: Vec::new(),
            }))
        }
    }

    pub fn new() -> Self {
        Self::new_with_config(AppConfig::default())
    }

    pub fn on_event(mut self, action: impl Fn(AppEvent) + 'static) -> Self {
        let inner : ApplicationInner = Arc::into_inner(self.inner).unwrap().into_inner();
        self.inner = Arc::new(RefCell::new(inner.on_event(action)));
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
        let inner : ApplicationInner = Arc::into_inner(self.inner).unwrap().into_inner();
        self.inner = Arc::new(RefCell::new(inner.window::<V>(app_view, config)));
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
        self.inner.borrow_mut().event_processing::<F, USER_EVENTS>(event_loop, f);
    }
}

/// Baseview gives nothing to grab hold of to easily figure out *which* window is being painted except the
/// identity of the handler being called (well, we could use the raw window handle as identity, but that
/// doesn't seem immensely reliable).
pub(crate) struct OneWindowHandler {
    pub window : WindowIdentifier,
    pub app : Arc<RefCell<ApplicationInner>>,
}

impl WindowHandler for OneWindowHandler {
    fn on_frame(&mut self, window: &mut baseview::Window) {
        self.app.borrow_mut().on_frame(&self.window, window)
    }

    fn on_event(&mut self, window: &mut baseview::Window, event: baseview::Event) -> baseview::EventStatus {
        self.app.borrow_mut().on_baseview_event(&self.window, window, event)
    }
}

pub(crate) struct ApplicationInner {
    pub(crate) receiver: Receiver<UserEvent>,
    pub(crate) handle: ApplicationHandle,
    pub(crate) initial_windows: Vec<WindowCreation>,
}

impl ApplicationInner {
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

    fn on_frame(&mut self, id : &WindowIdentifier, window: &mut baseview::Window) {
        let hack = BaseviewPseudoEventLoop::from(window);
        self.event_processing::<_, true>(&hack, move |handle, w| {
            handle.handle_updates_for_one_window(id, &hack);
        });
    }

    fn on_baseview_event(&mut self, id : &WindowIdentifier, window: &mut baseview::Window, event: baseview::Event) -> baseview::EventStatus {
        let hack = BaseviewPseudoEventLoop::from(window);
        self.event_processing::<_, false>(&hack, move |handle, window_opt| {
            handle.handle_window_event(id.to_owned(), event, window_opt);
        });
        baseview::EventStatus::Captured // pending, do we know?
    }
}
