use crate::{app_events::UserEvent, application::{
        app_handle::ApplicationHandle, baseview_hacks::{setting_current_window, BaseviewPseudoEventLoop}, spi::AppHandlerInternalAPI
    }, window::{WindowConfig, WindowCreation}, AppConfig, AppEvent, IntoView
};
use super::app::*;
use baseview::{Window, WindowHandler, WindowOpenOptions};
use floem_reactive::Runtime;
use windowing::public_api::WindowIdentifier;

#[cfg(feature = "crossbeam")]
use crossbeam::channel::{unbounded as channel};

#[cfg(not(feature = "crossbeam"))]
use std::sync::mpsc::{Receiver, Sender, channel};

#[cfg(feature = "crossbeam")]
use crossbeam::channel::{Receiver, Sender};

use std::{cell::RefCell, sync::Arc, thread};
use crate::AnyView;

type EventLoopType = <ApplicationHandle as AppHandlerInternalAPI>::WindowingSystemEventLoop;

/*
For baseview, we need to create listeners over an Application which can obtain a mutable reference
to it, and they need to know which window id they reference.  So, we bury the actual implementation
in an Arc<RefCell>.
*/

pub struct Application {
    pub(super) inner : ApplicationInner,
}

thread_local! {
    static ID_TRANSFER_HACK : RefCell<Option<(WindowIdentifier, windowing::public_api::BaseviewHandles)>> = RefCell::new(None);
}

fn create_one_window(opts : WindowOpenOptions, inner : ApplicationInner, view_fn: Box<dyn FnOnce(WindowIdentifier) -> AnyView>) {
    let mut info : Option<(WindowIdentifier, windowing::public_api::BaseviewHandles)> = None;
    let placeholder = WindowIdentifier::default();

    let mut listener = OneWindowHandler {
        window: placeholder,
        app: inner.clone(),
    };

    Window::open_blocking(opts, |win| {
        let (window_id, inner) = windowing::public_api::register_window(win);
        let ifo:Option<(WindowIdentifier, windowing::public_api::BaseviewHandles)> = Some((window_id.clone(), inner.clone()));
        ID_TRANSFER_HACK.with(|v| {
            v.replace(ifo);
        });
        listener.window = window_id;
        // listener.app.borrow_mut().handle.register_window(window_id, inner, view_fn);
        listener
    });
    if let Some((id, handles)) = ID_TRANSFER_HACK.take() {
        inner.handle.borrow_mut().register_window(id, handles, view_fn);
    }
}

impl Application {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn run(mut self) {
        Self::on_before_run();
        for w in self.inner.initial_windows() {
            let opts : baseview::WindowOpenOptions = w.config.map(baseview::WindowOpenOptions::from).unwrap_or(WindowOpenOptions {
                title: "Floem window".into(),
                size: baseview::Size::new(512., 512.),
                scale: baseview::WindowScalePolicy::SystemScaleFactor,
                gl_config: None,
            });
            let copy = self.inner.clone();
            create_one_window(opts, copy, w.view_fn);
        }
    }

    pub(crate) fn send_proxy_event(event: UserEvent) {
        todo!("Send proxy event not implemented: {:?}", event);
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
            inner : ApplicationInner {
                receiver: Arc::new(receiver),
                handle: Arc::new(RefCell::new(handle)),
                initial_windows: Arc::new(RefCell::new(Vec::new())),
            }
        }
    }

    pub fn new() -> Self {
        Self::new_with_config(AppConfig::default())
    }

    pub fn on_event(mut self, action: impl Fn(AppEvent) + 'static) -> Self {
        self.inner = self.inner.on_event(action);
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
        self.inner = self.inner.window::<V>(app_view, config);
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
        self.inner.event_processing::<F, USER_EVENTS>(event_loop, f);
    }
}

/// Baseview gives nothing to grab hold of to easily figure out *which* window is being painted except the
/// identity of the handler being called (well, we could use the raw window handle as identity, but that
/// doesn't seem immensely reliable).
pub(crate) struct OneWindowHandler {
    pub window : WindowIdentifier,
    pub app : ApplicationInner,
}

unsafe impl Send for OneWindowHandler{}
unsafe impl Sync for OneWindowHandler{}

impl WindowHandler for OneWindowHandler {
    fn on_frame(&mut self, window: &mut baseview::Window) {
        // self.app.borrow_mut().on_frame(&self.window, window)
        self.app.on_frame(&self.window, window);
    }

    fn on_event(&mut self, window: &mut baseview::Window, event: baseview::Event) -> baseview::EventStatus {
        println!("ON EVENT {:?}", event);
        self.app.on_baseview_event(&self.window, window, event);
        baseview::EventStatus::Ignored
    }
}

#[derive(Clone)]
pub(crate) struct ApplicationInner {
    pub(crate) receiver: Arc<Receiver<UserEvent>>,
    pub(crate) handle: Arc<RefCell<ApplicationHandle>>,
    pub(crate) initial_windows: Arc<RefCell<Vec<WindowCreation>>>,
}

impl ApplicationInner {

    fn initial_windows(&mut self) -> Vec<WindowCreation> {
        let mut v : Arc<RefCell<Vec<WindowCreation>>> = Arc::new(RefCell::new(vec![]));
        std::mem::swap(&mut v, &mut self.initial_windows);
        let r : RefCell<Vec<WindowCreation>> = Arc::into_inner(v).unwrap();
        let v : Vec<WindowCreation> = r.into_inner();
        return v
    }

    pub fn on_event(mut self, action: impl Fn(AppEvent) + 'static) -> Self {
        self.handle.borrow_mut().event_listener = Some(Box::new(action));
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
        self.initial_windows.borrow_mut().push(WindowCreation {
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
        self.handle.borrow_mut().handle_timer(event_loop);
        f(&mut self.handle.borrow_mut(), event_loop);
        if USER_EVENTS {
            let events : Vec<UserEvent> = self.receiver.as_ref().try_iter().collect();
            for event in events {
                self.handle.borrow_mut().handle_user_event(event_loop, event);
            }
        }
        if Runtime::has_pending_work() {
            Runtime::drain_pending_work();
        }
    }

    fn on_frame(&mut self, id : &WindowIdentifier, window: &mut baseview::Window) {
        let hack = BaseviewPseudoEventLoop::from(window);
        setting_current_window(hack, || {
            self.event_processing::<_, true>(&hack, move |handle, w| {
                handle.handle_updates_for_one_window(id, &hack);
            });
        })
    }

    fn on_baseview_event(&mut self, id : &WindowIdentifier, window: &mut baseview::Window, event: baseview::Event) -> baseview::EventStatus {
        let hack = BaseviewPseudoEventLoop::from(window);
        setting_current_window(hack, || {
            self.event_processing::<_, false>(&hack, move |handle, window_opt| {
                handle.handle_window_event(id.to_owned(), event, window_opt);
            });
        });
        baseview::EventStatus::Captured // pending, do we know?
    }
}
