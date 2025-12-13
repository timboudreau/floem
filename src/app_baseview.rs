use crate::application::spi::AppHandlerInternalAPI;

use super::app::*;
use baseview::WindowHandler;
use windowing::public_api::WindowIdentifier;

#[cfg(feature = "crossbeam")]
use crossbeam::channel::{unbounded as channel};

#[cfg(not(feature = "crossbeam"))]
use std::sync::mpsc::{Sender, channel};

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
            receiver,
            handle,
            initial_windows: Vec::new(),
        }
    }

    fn on_frame(&mut self, id : &WindowIdentifier, window: &mut baseview::Window) {

    }

    fn on_event(&mut self, id : &WindowIdentifier, window: &mut baseview::Window, event: baseview::Event) -> baseview::EventStatus {
        self.handle.handle_window_event(window_id, event, Some(window));
    }
}

struct OneWindowHandler {
    window : WindowIdentifier,
    app : Arc<RefCell<Application>>,
}

impl WindowHandler for OneWindowHandler {
    fn on_frame(&mut self, window: &mut baseview::Window) {
        self.app.borrow_mut().on_frame(&self.window, window)
    }

    fn on_event(&mut self, window: &mut baseview::Window, event: baseview::Event) -> baseview::EventStatus {
        self.app.borrow_mut().on_event(&self.window, window, event)
    }
}
