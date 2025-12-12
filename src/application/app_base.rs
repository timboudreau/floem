use std::rc::Rc;
use adapters::WindowSystemTheme;
use floem_reactive::{SignalUpdate, WriteSignal};
use muda::MenuId;
use windowing::public_api::WindowIdentifier;
use crate::{action::{Timer, TimerToken}, app_events::{AppUpdateEvent, UserEvent}, inspector::Capture, profiler::Profile, window::{WindowConfig, WindowCreation}, windows::window_handle::WindowHandle, AppConfig, AppEvent, View};

/// Contains the signatures the rest of floem may call on ApplicationHandle.
pub(crate) trait AppHandlerInternalAPI {
    type WindowingSystemEventLoop : ?Sized;
    type WindowingSystemWindowEvent;

    fn new(config: AppConfig) -> Self;
    fn new_window(&mut self, event_loop: &Self::WindowingSystemEventLoop, view_fn: Box<dyn FnOnce(WindowIdentifier) -> Box<dyn View>>,
        override_theme: Option<WindowSystemTheme>, config : WindowConfig);
    fn idle(&mut self);
    fn handle_updates_for_all_windows(&mut self);
    fn handle_timer(&mut self, event_loop: &Self::WindowingSystemEventLoop);
    fn handle_user_event(&mut self, event_loop: &Self::WindowingSystemEventLoop, event: UserEvent);
    fn handle_window_event(&mut self, window_id: WindowIdentifier, event: Self::WindowingSystemWindowEvent, event_loop: &Self::WindowingSystemEventLoop);
    fn window_handle_for_window_id(&mut self, window_id : &WindowIdentifier) -> Option<&mut WindowHandle>;
}

/// Internal API for ApplicationHandle which defines a contract both the winit and baseview implementations
/// must implement.  Any logic that can be shared across all implementations is implemented here rather than
/// duplicated.
pub(super) trait AppHandlerImpl : AppHandlerInternalAPI {
    fn close_window(&mut self, window_id: WindowIdentifier, event_loop: &Self::WindowingSystemEventLoop);
    fn capture_window(&mut self, window_id: WindowIdentifier) -> Option<Capture>;
    fn request_timer(&mut self, timer: Timer, event_loop: &Self::WindowingSystemEventLoop);
    fn remove_timer(&mut self, timer: &TimerToken, event_loop: &Self::WindowingSystemEventLoop);
    fn fire_timer(&mut self, event_loop: &Self::WindowingSystemEventLoop);
    fn handle_gpu_resource_update(&mut self, window_id : WindowIdentifier);
    fn handle_exit(&mut self, event_loop: &Self::WindowingSystemEventLoop);
    fn publish_event(&mut self, event : AppEvent);
    fn handle_menu_action(&mut self, action_id : MenuId);
    fn handle_theme_change(&mut self, theme : WindowSystemTheme);
    fn handle_window_creation(&mut self, creation: WindowCreation, event_loop: &Self::WindowingSystemEventLoop);
    fn handle_profile(&mut self, window_id: WindowIdentifier, end_profile: Option<WriteSignal<Option<Rc<Profile>>>>);

    fn handle_update_event(&mut self, event_loop: &Self::WindowingSystemEventLoop) {
        let events = crate::app_events::retreive_app_update_events();
        for event in events {
            match event {
                AppUpdateEvent::NewWindow { window_creation } => self.handle_window_creation(window_creation, event_loop),
                AppUpdateEvent::CloseWindow { window_id } => {
                    self.close_window(window_id, event_loop);
                }
                AppUpdateEvent::RequestTimer { timer } => {
                    self.request_timer(timer, event_loop);
                }
                AppUpdateEvent::CancelTimer { timer } => {
                    self.remove_timer(&timer, event_loop);
                }
                AppUpdateEvent::CaptureWindow { window_id, capture } => {
                    capture.set(self.capture_window(window_id).map(Rc::new));
                }
                AppUpdateEvent::ProfileWindow {
                    window_id,
                    end_profile,
                } => {
                    self.handle_profile(window_id, end_profile);
                }
                AppUpdateEvent::MenuAction { action_id } => {
                    self.handle_menu_action(action_id);
                }
                AppUpdateEvent::ThemeChanged { theme } => {
                    self.handle_theme_change(theme)
                }
            }
        }
    }

    fn internal_handle_user_event(&mut self, event_loop: &Self::WindowingSystemEventLoop, event: UserEvent) {
        match event {
            UserEvent::AppUpdate => {
                self.handle_update_event(event_loop);
            }
            UserEvent::Idle => {
                self.idle();
            }
            UserEvent::QuitApp => {
                self.handle_exit(event_loop);
            }
            UserEvent::Reopen {
                has_visible_windows,
            } => {
                self.publish_event(AppEvent::Reopen {
                    has_visible_windows,
                });
            }
            UserEvent::GpuResourcesUpdate { window_id } => {
                self.handle_gpu_resource_update(window_id);
            }
        }
    }
}
