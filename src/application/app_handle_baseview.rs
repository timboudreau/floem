use super::{
    app_handle::{ApplicationHandle, *},
    spi::{AppHandlerCommon, AppHandlerImpl, AppHandlerInternalAPI},
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
};
use adapters::WindowSystemTheme;
use floem_reactive::{SignalUpdate, WriteSignal};
use muda::MenuId;
use std::rc::Rc;
use ui_events_baseview::WindowEventTranslation;
use windowing::public_api::WindowIdentifier;

impl AppHandlerInternalAPI for ApplicationHandle {
    type WindowingSystemEventLoop = ();

    type WindowingSystemWindowEvent = ();

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

    fn new_window(
        &mut self,
        event_loop: &Self::WindowingSystemEventLoop,
        view_fn: Box<dyn FnOnce(WindowIdentifier) -> Box<dyn crate::View>>,
        override_theme: Option<adapters::WindowSystemTheme>,
        config: crate::window::WindowConfig,
    ) {
        todo!()
    }

    fn handle_updates_for_all_windows(&mut self) {
        todo!()
    }

    fn handle_timer(&mut self, event_loop: &Self::WindowingSystemEventLoop) {
        todo!()
    }

    fn handle_user_event(
        &mut self,
        event_loop: &Self::WindowingSystemEventLoop,
        event: crate::app_events::UserEvent,
    ) {
        todo!()
    }

    fn handle_window_event(
        &mut self,
        window_id: WindowIdentifier,
        event: Self::WindowingSystemWindowEvent,
        event_loop: &Self::WindowingSystemEventLoop,
    ) {
        todo!()
    }
}

impl AppHandlerImpl for ApplicationHandle {
    fn close_window(
        &mut self,
        window_id: WindowIdentifier,
        event_loop: &Self::WindowingSystemEventLoop,
    ) {
        todo!()
    }

    fn request_timer(
        &mut self,
        timer: crate::action::Timer,
        event_loop: &Self::WindowingSystemEventLoop,
    ) {
        todo!()
    }

    fn remove_timer(
        &mut self,
        timer: &crate::action::TimerToken,
        event_loop: &Self::WindowingSystemEventLoop,
    ) {
        todo!()
    }

    fn fire_timer(&mut self, event_loop: &Self::WindowingSystemEventLoop) {
        todo!()
    }

    fn handle_gpu_resource_update(&mut self, window_id: WindowIdentifier) {
        todo!()
    }

    fn handle_exit(&mut self, event_loop: &Self::WindowingSystemEventLoop) {
        todo!()
    }
}
