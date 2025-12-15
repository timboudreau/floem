use crate::{
    app_baseview_events::create_one_window,
    app_events::UserEvent,
    application::{
        app_handle::ApplicationHandle,
        baseview_hacks::{setting_current_window, BaseviewPseudoEventLoop},
        spi::AppHandlerInternalAPI,
    },
    window::{WindowConfig, WindowCreation},
    AppConfig, AppEvent, IntoView,
};
use baseview::WindowOpenOptions;
use floem_reactive::Runtime;
use parking_lot::Mutex;
use windowing::public_api::WindowIdentifier;
use std::{cell::RefCell, sync::Arc};

#[cfg(not(feature = "crossbeam"))]
use std::sync::mpsc::{channel, Receiver, Sender};

#[cfg(feature = "crossbeam")]
use crossbeam::channel::{Receiver, Sender, unbounded as channel};

type EventLoopType = <ApplicationHandle as AppHandlerInternalAPI>::WindowingSystemEventLoop;

/*
For baseview, we need to create listeners over an Application which can obtain a mutable reference
to it, and they need to know which window id they reference.  So, we bury the actual implementation
in an Arc<RefCell>.
*/

#[repr(transparent)]
pub struct Application {
    pub(super) inner: ApplicationInner,
}

static PENDING_USER_EVENTS: Mutex<Option<Sender<UserEvent>>> = Mutex::new(None);

impl Application {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn run(mut self) {
        Self::on_before_run();
        for w in self.inner.initial_windows() {
            let opts: baseview::WindowOpenOptions = w
                .config
                .map(baseview::WindowOpenOptions::from)
                .unwrap_or(WindowOpenOptions {
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
        if let Some(sender) = PENDING_USER_EVENTS.lock().as_ref() {
            // println!("Enqueue proxy event: {:?}", event);
            let _ = sender.send(event);
        }
    }

    pub fn new_with_config(config: AppConfig) -> Self {
        crate::screen_layout::ensure_windowing_system_initialized();

        #[cfg(target_os = "macos")]
        crate::app_delegate::set_app_delegate();

        let (sender, receiver) = channel();

        *PENDING_USER_EVENTS.lock() = Some(sender);

        // Pending - any baseview clipboard init
        let handle = ApplicationHandle::new(config);

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
            use crate::app_events::{add_app_update_event, AppUpdateEvent};

            add_app_update_event(AppUpdateEvent::MenuAction {
                action_id: event.id,
            });
        }));
        Self {
            inner: ApplicationInner {
                receiver: Arc::new(receiver),
                handle: Arc::new(RefCell::new(handle)),
                initial_windows: Arc::new(RefCell::new(Vec::new())),
            },
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
}

#[derive(Clone)]
pub(crate) struct ApplicationInner {
    // These need to be separately borrowable, so we can't just have
    // Application { inner : Arc<...> } or some call sequences will be impossible.
    //
    // Technically these could be fields of Application, but keeping the API separate
    // from the implementation avoids inadvertently exposing stuff that shouldn't be.
    pub(crate) receiver: Arc<Receiver<UserEvent>>,
    pub(crate) handle: Arc<RefCell<ApplicationHandle>>,
    pub(crate) initial_windows: Arc<RefCell<Vec<WindowCreation>>>,
}

// This cloning may be expensive.
/// We ephemerally set this thread local to the the application contents, so that
/// it can be retrieved for creating baseview event listeners, which need to call it.
/// Since the call to create the initial window does not return, there is no way
/// to make the Application instance accessible to attach to otherwise - it consumes
/// Application.
#[inline(always)]
fn setting_app_inner<F: FnOnce() -> T, T>(clone: ApplicationInner, f: F) -> T {
    crate::app_baseview_events::APP_INNER.with(|cell| {
        *cell.borrow_mut() = Some(clone);
        let result = f();
        *cell.borrow_mut() = None;
        result
    })
}

impl ApplicationInner {
    /// Drain the initial windows configured before the application was started.
    fn initial_windows(&mut self) -> Vec<WindowCreation> {
        let mut v: Arc<RefCell<Vec<WindowCreation>>> = Arc::new(RefCell::new(vec![]));
        std::mem::swap(&mut v, &mut self.initial_windows);
        let r: RefCell<Vec<WindowCreation>> = Arc::into_inner(v).unwrap();
        let v: Vec<WindowCreation> = r.into_inner();
        return v;
    }

    pub fn on_event(self, action: impl Fn(AppEvent) + 'static) -> Self {
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
        self,
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
    pub(super) fn event_processing<
        F: FnOnce(&mut ApplicationHandle, &EventLoopType),
        const USER_EVENTS: bool,
    >(
        &mut self,
        event_loop: &EventLoopType,
        f: F,
    ) {
        setting_app_inner(self.clone(), || {
            self.handle.borrow_mut().handle_timer(event_loop);
            f(&mut self.handle.borrow_mut(), event_loop);
            if USER_EVENTS {
                let events: Vec<UserEvent> = self.receiver.as_ref().try_iter().collect();
                for event in events {
                    self.handle
                        .borrow_mut()
                        .handle_user_event(event_loop, event);
                }
            }
            if Runtime::has_pending_work() {
                Runtime::drain_pending_work();
            }
        });
        // We need to do this here, or we will be borrowed at the time the listener fires and unable to
        // get a mutable reference to the handle to create a window handle for a newly opened *child*
        // window.  It's ugly, but works.
        if let Some((window_id, handles, view_fn)) =
            super::app_baseview_events::DEFERRED_REGISTER.take()
        {
            self.handle
                .borrow_mut()
                .register_window(window_id, handles, view_fn);
        }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub(super) fn on_frame(&mut self, id: &WindowIdentifier, window: &mut baseview::Window) {
        println!("ON FRAME {:#x} or {:#b} {}", id.id(), id.id(), id.id());
        // We create an unsafe pointer to the window that allows parts of the public API that have no access
        // to it to have minimal access to manipulate it.  Where possible, we simply pass it in place of the
        // &ActiveEventLoop from the winit implementation, but in some cases that is impossible without more
        // invasive changes.
        let hack = BaseviewPseudoEventLoop::from(window);
        setting_current_window(hack, || {
            self.event_processing::<_, true>(&hack, move |handle, _w| {
                handle.handle_updates_for_one_window(id, &hack);
                // XXX should only be the current window
                handle.render_one(id, &hack);
            });
        })
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub(super) fn on_baseview_event(
        &mut self,
        id: &WindowIdentifier,
        window: &mut baseview::Window,
        event: baseview::Event,
    ) -> baseview::EventStatus {
        // We create an unsafe pointer to the window that allows parts of the public API that have no access
        // to it to have minimal access to manipulate it.  Where possible, we simply pass it in place of the
        // &ActiveEventLoop from the winit implementation, but in some cases that is impossible without more
        // invasive changes.
        let hack = BaseviewPseudoEventLoop::from(window);
        setting_current_window(hack, || {
            self.event_processing::<_, false>(&hack, move |handle, window_opt| {
                handle.handle_window_event(id.to_owned(), event, window_opt);
            });
        });
        baseview::EventStatus::Captured // pending, do we know?
    }
}
