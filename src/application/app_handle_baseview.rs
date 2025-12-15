use super::{
    app_handle::ApplicationHandle,
    baseview_hacks::BaseviewPseudoEventLoop,
    spi::{AppHandlerImpl, AppHandlerInternalAPI},
};
use crate::{
    action::{TimerToken},
    app_baseview_events::{on_before_attach_new_child_window, LateRegisteringWindowHandler},
    application::baseview_hacks::current_window,
    context::PaintState,
    kurbo::Size,
    windows::window_handle::WindowHandle, View
};
use baseview::*;
use ui_events::pointer::PointerEvent;
use ui_events_baseview::WindowEventTranslation;
use windowing::public_api::WindowIdentifier;

impl ApplicationHandle {
    pub(crate) fn register_window<F: FnOnce(WindowIdentifier) -> Box<dyn View> + 'static>(
        &mut self,
        id: WindowIdentifier,
        handles: windowing::public_api::NativeWindowInner,
        view_fn: F,
    ) {
        println!("Register window {:?} for {:?}", id, handles);
        let handle = WindowHandle::new(
            Box::new(handles),
            None,
            Default::default(),
            view_fn,
            false,
            false,
            1.,
        );
        self.window_handles.insert(id, handle);
    }
}

impl AppHandlerInternalAPI for ApplicationHandle {
    type WindowingSystemEventLoop = BaseviewPseudoEventLoop;
    type WindowingSystemWindowEvent = Event;

    fn handle_window_creation(
        &mut self,
        creation: crate::window::WindowCreation,
        event_loop: &Self::WindowingSystemEventLoop,
    ) {
        self.new_window(
            event_loop,
            creation.view_fn,
            None,
            creation.config.unwrap_or_default(),
        );
    }

    fn new(config: crate::AppConfig) -> Self {
        Self::from(config)
    }

    fn remove_timer(&mut self, timer: &TimerToken, _event_loop: &Self::WindowingSystemEventLoop) {
        self.timers.remove(timer);
    }

    fn new_window(
        &mut self,
        event_loop: &Self::WindowingSystemEventLoop,
        view_fn: Box<dyn FnOnce(WindowIdentifier) -> Box<dyn crate::View>>,
        _override_theme: Option<adapters::WindowSystemTheme>,
        config: crate::window::WindowConfig,
    ) {
        let opts = baseview::WindowOpenOptions {
            title: config.title,
            size: baseview::Size {
                width: config.size.unwrap_or(Size::new(512., 512.)).width,
                height: config.size.unwrap_or(Size::new(512., 512.)).height,
            },
            scale: baseview::WindowScalePolicy::SystemScaleFactor, // pending, set?
            gl_config: None,                                       // XXX use in some cases?
        };

        // Stores the view_fn in a thread_local so LateRegisteringWindowHandler::new() can grab it
        // without the compiler complaining that it can't be moved into the callback below (if we
        // are running on some other thread, it will find nothing there and panic; but also, this
        // method can only be called from the event thread in a window callback, and the same goes
        // [at least on mac os?] for window creation).
        on_before_attach_new_child_window(view_fn);
        let _handle = event_loop.with_ref(|parent_window| {
            Window::open_parented(parent_window, opts, |child_window| {
                // log the window address so we can diagnose painting the wrong window
                println!("In callback for creating child window {:?}", BaseviewPseudoEventLoop::from(child_window));
                LateRegisteringWindowHandler::new()
            })
        });

        // We might just be able to do this deriving BaseviewHandles from the returned WindowHandle.
        //
        // Nope, they only have a window handle, not a display handle, and we need both (though Mac OS
        // DisplayHandle is a no-op).
    }

    fn handle_updates_for_all_windows(&mut self) {
        // unreachable!("This method cannot be implemented for baseview and should not be reachable.");
        let ww = current_window();
        if !ww.is_none() {
            // println!("handle_updates_for_all_windows gets a hacked window {:?}", ww);
            let reg_id: Option<WindowIdentifier> = ww
                .with_mut(|r| windowing::public_api::find(r))
                .unwrap_or(None);
            if let Some(id) = reg_id {
                // println!("Will try to process updates for this window as {:?}", id);
                self.handle_updates_for_one_window(&id, &ww);
            }
        }
    }

    fn handle_timer(&mut self, event_loop: &Self::WindowingSystemEventLoop) {
        self.internal_handle_timer(event_loop);
    }

    fn handle_user_event(
        &mut self,
        event_loop: &Self::WindowingSystemEventLoop,
        event: crate::app_events::UserEvent,
    ) {
        self.internal_handle_user_event(event_loop, event);
    }

    #[cfg_attr(debug_assertions, track_caller)]
    fn handle_window_event(
        &mut self,
        window_id: WindowIdentifier,
        event: Event,
        _event_loop: &Self::WindowingSystemEventLoop,
    ) {
        let window_handle = match self.window_handles.get_mut(&window_id) {
            Some(window_handle) => window_handle,
            None => return,
        };

        match window_handle
            .event_reducer
            .reduce(window_handle.scale, &event)
        {
            Some(WindowEventTranslation::Keyboard(ke)) => {
                println!("Forward xlated key event : {:?}", ke);
                window_handle.key_event(ke);
                // FIXME:
                // if let WindowEvent::KeyboardInput { is_synthetic, .. } = event {
                //     if !is_synthetic {
                //         window_handle.key_event(ke)
                //     }
                // }
            }
            Some(WindowEventTranslation::Pointer(pe)) => {
                if matches![pe, PointerEvent::Down(_)] {
                    println!("Forward xlated pointer down event: {:?}", pe);
                }
                window_handle.pointer_event(pe);
            }
            None => {}
        }

        match event {
            Event::Mouse(_mouse_event) => {
                // pending
            }
            Event::Keyboard(_keyboard_event) => {
                // pending
            }
            Event::Window(window_event) => {
                match window_event {
                    WindowEvent::Resized(window_info) => {
                        println!("Got window resized info: {:?}", window_info);
                        let size = Size::new(
                            window_info.logical_size().width,
                            window_info.logical_size().height,
                        );
                        window_handle.size(size);
                    }
                    WindowEvent::Focused => {
                        println!("Got window focused");
                        window_handle.focused(true);
                    }
                    WindowEvent::Unfocused => {
                        println!("Got window unfocused");
                        window_handle.focused(false);
                    }
                    WindowEvent::WillClose => {
                        println!("Got window will-close");
                        // todo!()
                    }
                }
            }
        }
    }
}

impl AppHandlerImpl for ApplicationHandle {
    fn close_window(
        &mut self,
        window_id: WindowIdentifier,
        event_loop: &Self::WindowingSystemEventLoop,
    ) {
        if let Some(handle) = self.window_handles.get_mut(&window_id) {
            handle.destroy();
        }
        self.window_handles.remove(&window_id);
        event_loop.close();
        if self.window_handles.is_empty() && self.config.exit_on_close {
            self.handle_exit(event_loop);
        }
    }

    fn request_timer(
        &mut self,
        timer: crate::action::Timer,
        event_loop: &Self::WindowingSystemEventLoop,
    ) {
        self.timers.insert(timer.token, timer);
        self.fire_timer(event_loop);
    }

    fn fire_timer(&mut self, _event_loop: &Self::WindowingSystemEventLoop) {
        // do nothing
    }

    fn handle_gpu_resource_update(&mut self, window_id: WindowIdentifier) {
        let handle = self
            .window_handles
            .get_mut(&window_id)
            .expect("No window handle for id");
        if let PaintState::PendingGpuResources {
            window,
            rx,
            font_embolden,
            renderer,
        } = &handle.paint_state
        {
            let (gpu_resources, surface) = rx.recv().unwrap().unwrap();
            let renderer = crate::renderer::Renderer::new(
                window.clone(),
                gpu_resources.clone(),
                surface,
                renderer.scale(),
                renderer.size(),
                *font_embolden,
            );
            self.gpu_resources = Some(gpu_resources);
            handle.paint_state = PaintState::Initialized { renderer };
            handle.init_renderer(self.gpu_resources.clone());
        } else {
            panic!("Sent a gpu resource update after it had already been initialized");
        }
    }

    fn handle_exit(&mut self, _event_loop: &Self::WindowingSystemEventLoop) {
        // ?
        std::process::exit(0)
    }
}
