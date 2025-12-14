use super::{
    app_handle::{ApplicationHandle, *},
    spi::{AppHandlerCommon, AppHandlerImpl, AppHandlerInternalAPI},
    baseview_hacks::BaseviewPseudoEventLoop,
};
use crate::{
    action::{Timer, TimerToken},
    app_events::{AppUpdateEvent, UserEvent},
    ext_event::EXT_EVENT_HANDLER,
    inspector::Capture,
    profiler::Profile,
    window::{WindowConfig, WindowCreation},
    windows::window_handle::WindowHandle,
    AppConfig, AppEvent, View,
    kurbo::Size,
};
use adapters::WindowSystemTheme;
use baseview::{*, gl::*};
use floem_reactive::{SignalUpdate, WriteSignal};
use muda::MenuId;
use std::rc::Rc;
use ui_events_baseview::WindowEventTranslation;
use windowing::public_api::WindowIdentifier;

impl ApplicationHandle {
    pub(crate) fn with_window_handle_for<F : FnOnce(&mut WindowHandle)>(&mut self, id : &WindowIdentifier, f : F) {
        if let Some(h) = self.window_handles.get_mut(id) {
            f(h)
        } else {
            panic!("No window handle for {:?}", id);
        }
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
        todo!()
    }

    fn new(config: crate::AppConfig) -> Self {
        todo!()
    }

    fn remove_timer(&mut self, timer: &TimerToken, event_loop: &Self::WindowingSystemEventLoop) {
        todo!()
    }

    fn new_window(
        &mut self,
        event_loop: &Self::WindowingSystemEventLoop,
        view_fn: Box<dyn FnOnce(WindowIdentifier) -> Box<dyn crate::View>>,
        override_theme: Option<adapters::WindowSystemTheme>,
        config: crate::window::WindowConfig,
    ) {
        let opts = baseview::WindowOpenOptions {
            title: config.title,
            size: baseview::Size {
                width : config.size.unwrap_or(Size::new(512., 512.)).width,
                height: config.size.unwrap_or(Size::new(512., 512.)).height,
            },
            scale: baseview::WindowScalePolicy::SystemScaleFactor, // pending, set?
            gl_config: None, // XXX use in some cases?
        };
        // baseview::Window::open_blocking(opts, |win| {

        // });

        todo!()
    }

    fn handle_updates_for_all_windows(&mut self) {
        unreachable!("This method cannot be implemented for baseview and should not be reachable.");
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

    fn handle_window_event(
        &mut self,
        window_id: WindowIdentifier,
        event: Event,
        event_loop: &Self::WindowingSystemEventLoop,
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
                window_handle.key_event(ke);
                // FIXME:
                // if let WindowEvent::KeyboardInput { is_synthetic, .. } = event {
                //     if !is_synthetic {
                //         window_handle.key_event(ke)
                //     }
                // }
            }
            Some(WindowEventTranslation::Pointer(pe)) => {
                window_handle.pointer_event(pe);
            }
            None => {}
        }

        match event {
            Event::Mouse(mouse_event) => {
                // pending
            },
            Event::Keyboard(keyboard_event) => {
                // pending
            },
            Event::Window(window_event) => {
                match window_event {
                    WindowEvent::Resized(window_info) => {
                        let size = Size::new(window_info.logical_size().width, window_info.logical_size().height);
                        window_handle.size(size);
                    },
                    WindowEvent::Focused => {
                        window_handle.focused(true);
                    },
                    WindowEvent::Unfocused => {
                        window_handle.focused(false);
                    },
                    WindowEvent::WillClose => {
                        todo!()
                    },
                }
            },
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

    fn fire_timer(&mut self, event_loop: &Self::WindowingSystemEventLoop) {
        // do nothing
    }

    fn handle_gpu_resource_update(&mut self, window_id: WindowIdentifier) {
        todo!()
    }

    fn handle_exit(&mut self, event_loop: &Self::WindowingSystemEventLoop) {
        // ?
        std::process::exit(0)
    }
}
