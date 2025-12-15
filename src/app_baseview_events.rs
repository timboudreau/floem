use crate::{
    app_baseview::ApplicationInner,
    application::baseview_hacks::{
        current_window, setting_current_window, BaseviewPseudoEventLoop,
    },
    AnyView, Clipboard,
};
use baseview::{Event, EventStatus, Window, WindowHandler, WindowOpenOptions};
use std::{cell::RefCell, sync::atomic::AtomicBool};
use windowing::public_api::*;

type ViewFn = Box<dyn FnOnce(WindowIdentifier) -> AnyView>;

static CLIPBOARD_INITIALIZED: AtomicBool = AtomicBool::new(false);
thread_local! {
    /// Set at the top of an event or paint callback, and cleared at the end of it.  Used to allow
    /// child window creation.
    pub(super) static APP_INNER : RefCell<Option<ApplicationInner>> = Default::default();

    /// Used so that the window creation closure, which must be `Send` can share enough information about the window
    /// for us to create a window handle for it.
    static ID_TRANSFER_HACK : RefCell<Option<(WindowIdentifier, windowing::public_api::BaseviewHandles)>> = RefCell::new(None);

    /// For child windows: They arrive without a valid window handle, but will have one by the first event or paint
    /// callback.  So we use a listener that intercepts the first callback, extracts enough information to create a
    /// window handle, create the real listener we will use, and replace itself with that.  It cannot actually register
    /// the window handle, because the application handle is borrowed at the time it is called, so it will put the
    /// needed information into ...
    static INNER_TRANSFER : RefCell<Option<(ApplicationInner, ViewFn)>> = RefCell::new(None);

    /// ApplicationInner's baseview callback methods will check this on each pass and, if non-empty, will register a window handle.
    pub(super) static DEFERRED_REGISTER : RefCell<Option<(WindowIdentifier, windowing::public_api::BaseviewHandles, ViewFn)>> = RefCell::new(None);
}

/// Baseview gives nothing to grab hold of to easily figure out *which* window is being painted except the
/// identity of the handler being called, so unlike the winit implementation, we need a separate listener for
/// each window that knows what window it is responding to.
pub(crate) struct OneWindowHandler {
    pub window: WindowIdentifier,
    pub app: ApplicationInner,
}

unsafe impl Send for OneWindowHandler {}
unsafe impl Sync for OneWindowHandler {}

impl WindowHandler for OneWindowHandler {
    #[cfg_attr(debug_assertions, track_caller)]
    fn on_frame(&mut self, window: &mut Window) {
        self.app.on_frame(&self.window, window);
    }

    #[cfg_attr(debug_assertions, track_caller)]
    fn on_event(&mut self, window: &mut Window, event: Event) -> EventStatus {
        // println!("ON EVENT {:?} id {:?}", event, self.window);
        self.app.on_baseview_event(&self.window, window, event)
    }
}

/// baseview *child* windows have a null pointer for their window handle at the time
/// the creation callback runs; while `create_parented` does return a `baseview::WindowHandle`,
/// that does not include the `DisplayHandle` needed to create gpu resources for it (although,
/// for Mac OS the display handle is a content-free, zero sized type, perhaps it is not on other
/// platforms).  So, we instead create a listener that, on the first event callback, morphs
/// itself into a real listener and proceeds along its merry way.
pub(crate) enum LateRegisteringWindowHandler {
    /// Retrieves handles from the window and replaces self with the real listener
    AwaitingRegistration(Option<ViewFn>, ApplicationInner),
    /// Delegates to its inner `OneWindowHandler`
    Ready(OneWindowHandler),
}

impl LateRegisteringWindowHandler {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn new() -> Self {
        // The code that creates child windows *must* pre-populate INNER_TRANSFER (non-thread-safe contents cannot
        // be passed into the closure that creates the window).
        INNER_TRANSFER.with(|it| {
            if let Some((inner, view_fn)) = it.replace(None) {
                Self::AwaitingRegistration(Some(view_fn), inner)
            } else {
                panic!("on_before_attach_new_child_window must be called immediately prior to LateRegisteringWindowHandler::new() on the same thread.");
            }
        })
    }

    /// Called the one and only type `Self::AwaitingRegistration` is invoked, to register the window,
    /// store enough information to create a window handle later, and replace `self` with `Self::Ready`.
    #[cfg_attr(debug_assertions, track_caller)]
    fn on_first_call(&mut self, window: &mut Window, view_fn: ViewFn, inner: ApplicationInner) {
        println!("Register the window");

        let (id, handles) = windowing::public_api::register_window(window);

        println!("Got {:?}, {:?} from registering the window.  Now call AppHandle to create *our* WindowHandle.", id, handles);

        // Okay, and here we are *still* unable to use handles.
        let old = DEFERRED_REGISTER.replace(Some((id, handles, view_fn)));
        debug_assert!(old.is_none(), "Unconsumed window registration info found");

        let result = Self::Ready(OneWindowHandler {
            window: id,
            app: inner,
        });
        println!("Replacing late registering listener now. Id: {:?}", id);
        *self = result
    }
}

impl WindowHandler for LateRegisteringWindowHandler {
    #[cfg_attr(debug_assertions, track_caller)]
    fn on_frame(&mut self, window: &mut Window) {
        let hack = BaseviewPseudoEventLoop::from(&mut *window);
        setting_current_window(hack, move || {
            let mut innards: Option<(ViewFn, ApplicationInner)>;
            match self {
                // We have to do the swapping of self for a Ready instance after registering outside of
                // this match block, since we have to reassign self.
                LateRegisteringWindowHandler::AwaitingRegistration(view_fn, inner) => {
                    let view_fn = view_fn
                        .take()
                        .expect("Late registration called twice - frame");
                    println!("LateRegistering convert on paint");
                    innards = Some((view_fn, inner.clone()));
                }
                LateRegisteringWindowHandler::Ready(delegate) => {
                    current_window().with_mut(|m| delegate.on_frame(m));
                    return;
                }
            }
            if let Some((view_fn, inner)) = innards.take() {
                self.on_first_call(window, view_fn, inner);
                //
                // Sigh. We cannot process the first event, because on_first_call will fail if it tries
                // to borrow_mut() the app handle.
                //
                // We are now calling Self::Ready.
                // self.on_frame(window);
            } else {
                panic!("Event: Should not be called in AwaitingRegistration state more than once.");
            }
        })
    }

    #[cfg_attr(debug_assertions, track_caller)]
    fn on_event(&mut self, window: &mut Window, event: Event) -> EventStatus {
        let hack = BaseviewPseudoEventLoop::from(&mut *window);
        setting_current_window(hack, move || {
            let mut innards: Option<(ViewFn, ApplicationInner)>;

            match self {
                // We have to do the swapping of self for a Ready instance after registering outside of
                // this match block, since we have to reassign self.
                LateRegisteringWindowHandler::AwaitingRegistration(view_fn, inner) => {
                    let view_fn = view_fn
                        .take()
                        .expect("Late registration called twice - event");
                    println!("LateRegistering convert on event : {:?} with window {:?}", event, hack);
                    innards = Some((view_fn, inner.clone()));
                }
                LateRegisteringWindowHandler::Ready(delegate) => {
                    return current_window()
                        .with_mut(|m| delegate.on_event(m, event))
                        .expect("Window must be set");
                }
            }
            if let Some((view_fn, inner)) = innards.take() {
                println!("Have some innards, converting.");
                let _ = self.on_first_call(window, view_fn, inner);
                //
                // Sigh. We cannot process the first event, because on_first_call will fail if it tries
                // to borrow_mut() the app handle.
                //
                // We are now calling Self::Ready.
                // self.on_event(window, event)
                EventStatus::Ignored
            } else {
                panic!("Event: Should not be called in AwaitingRegistration state more than once.");
            }
        })
    }
}

/// Called by child window creation to store the callback in a thread-local that can be
/// accessed within the closure that creates the window without it being `Send` (this will
/// break if, on any platform, the callback actually comes on a different thread).
#[cfg_attr(debug_assertions, track_caller)]
pub(crate) fn on_before_attach_new_child_window(view_fn: ViewFn) {
    // We need to make a clone of `ApplicationInner`, currently stored in one thread-local,
    // so that a listener can be created over it.
    APP_INNER.with(|inner| {
        let inner = inner
            .borrow()
            .as_ref()
            .expect("Called outside an event callback")
            .clone();
        INNER_TRANSFER.set(Some((inner, view_fn)));
    });
}

fn maybe_initialize_clipboard(window: &windowing::public_api::BaseviewHandles) {
    match CLIPBOARD_INITIALIZED.compare_exchange(
        false,
        true,
        std::sync::atomic::Ordering::SeqCst,
        std::sync::atomic::Ordering::Relaxed,
    ) {
        Ok(old) => {
            if !old {
                unsafe {
                    Clipboard::init(window.display);
                }
            }
        }
        Err(_) => (),
    }
}

/// Initial window creation.
#[cfg_attr(debug_assertions, track_caller)]
pub(super) fn create_one_window(
    opts: WindowOpenOptions,
    inner: ApplicationInner,
    view_fn: Box<dyn FnOnce(WindowIdentifier) -> AnyView>,
) {
    let placeholder = WindowIdentifier::default();

    let mut listener = OneWindowHandler {
        window: placeholder,
        app: inner.clone(),
    };
    // Store these so they can be accessed within the closure.
    INNER_TRANSFER.with(|i| {
        i.replace(Some((inner, view_fn)));
    });

    // Well, shoot, this method never returns.
    Window::open_blocking(opts, |win| {
        let (window_id, inner) = windowing::public_api::register_window(win);
        maybe_initialize_clipboard(&inner);
        println!("Opened window as {:?}", window_id);
        setting_current_window(BaseviewPseudoEventLoop::from(win), || {
            let ifo: Option<(WindowIdentifier, windowing::public_api::BaseviewHandles)> =
                Some((window_id.clone(), inner.clone()));
            ID_TRANSFER_HACK.with(|v| {
                v.replace(ifo);
            });
            listener.window = window_id;
            register_the_window();
            listener
        })
    });
}

fn register_the_window() {
    if let Some((id, handles)) = ID_TRANSFER_HACK.take() {
        if let Some((inner, view_fn)) = INNER_TRANSFER.take() {
            inner
                .handle
                .borrow_mut()
                .register_window(id, handles, view_fn);
        } else {
            println!("NO INNER");
        }
    } else {
        println!("NO TRANSFER");
    }
}
