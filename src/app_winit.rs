//! winit-specific parts of Application

use adapters::WindowSystemTheme;
use floem_reactive::Runtime;
use parking_lot::Mutex;
use raw_window_handle::HasDisplayHandle;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy}, window::WindowId};
use crate::{app::Application, app_events::UserEvent, app_handle::ApplicationHandle, AppConfig, AppEvent, Clipboard};

#[cfg(feature = "crossbeam")]
use crossbeam::channel::{unbounded as channel};

#[cfg(not(feature = "crossbeam"))]
use std::sync::mpsc::{Sender, channel};

static EVENT_LOOP_PROXY: Mutex<Option<(EventLoopProxy, Sender<UserEvent>)>> = Mutex::new(None);

impl Application {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn run(mut self) {
        Self::on_before_run();
        let event_loop = self.event_loop.take().unwrap();
        let _ = event_loop.run_app(self);
    }

    pub(crate) fn send_proxy_event(event: UserEvent) {
        if let Some((proxy, sender)) = EVENT_LOOP_PROXY.lock().as_ref() {
            let _ = sender.send(event);
            proxy.wake_up();
        }
    }

    pub fn new_with_config(config: AppConfig) -> Self {
        crate::screen_layout::ensure_windowing_system_initialized();
        let event_loop = EventLoop::new().expect("can't start the event loop");

        #[cfg(target_os = "macos")]
        crate::app_delegate::set_app_delegate();

        let event_loop_proxy = event_loop.create_proxy();
        let (sender, receiver) = channel();

        *EVENT_LOOP_PROXY.lock() = Some((event_loop_proxy.clone(), sender));
        unsafe {
            Clipboard::init(event_loop.display_handle().unwrap().as_raw());
        }
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
            event_loop: Some(event_loop),
            initial_windows: Vec::new(),
        }
    }
}

impl ApplicationHandler for Application {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        while let Some(window_creation) = self.initial_windows.pop() {
            self.handle.new_window(
                event_loop,
                window_creation.view_fn,
                self.handle.config.global_theme_override.map(WindowSystemTheme::into),
                window_creation.config.unwrap_or_default(),
            );
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.handle.handle_timer(event_loop);
        self.handle
            .handle_window_event(window_id.into(), event, event_loop);
        if Runtime::has_pending_work() {
            Runtime::drain_pending_work();
        }
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.handle.handle_timer(event_loop);
        for event in self.receiver.try_iter() {
            self.handle.handle_user_event(event_loop, event);
        }
        self.handle.handle_updates_for_all_windows();
        if Runtime::has_pending_work() {
            Runtime::drain_pending_work();
        }
    }

    fn destroy_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {
        if let Some(action) = self.handle.event_listener.as_ref() {
            action(AppEvent::WillTerminate);
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.handle.handle_timer(event_loop);
        if Runtime::has_pending_work() {
            Runtime::drain_pending_work();
        }
    }
}
