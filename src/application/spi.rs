//! What this all is:
//! We have (at least) *two* implementations of `ApplicationHandle`, one for `winit` and one for `baseview`.
//! `ApplicationHandle` has an API - the set of its methods that were formerly `pub(crate)`.  Those are
//! provided via `AppHandlerInternalAPI`.
//!
//! `ApplicationHandler`'s internal methods which need separate per-windowing-system implementation are
//! implemented in `AppHandlerImpl`.
//!
//! Some methods can be implemented once for all windowing systems.  Those are in `AppHandlerCommon` and
//! an implementation is in `../app_handle.rs`.
//!
//! Implementing `ApplicationHandle` for another windowing framework is a question of implementing `AppHandlerInternalAPI`
//! and `AppHandlerImpl` to talk to that windowing framework (well, plus implementing a lot of other things).
//!
//! So, effectively, these traits define the service provider interface for `ApplicationHandle`, while attempting
//! to require little to no code-duplication.
//!
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
use windowing::public_api::WindowIdentifier;

/// Functionality that can be implemented once and shared for both winit and baseview, so we can get the surface
/// area of `AppHandlerInternalAPI` and `AppHandlerImpl` down to *just* that which needs to be uniquely implemented for each.
///
/// These are generally methods that touch some fields of `ApplicationHandle`, but not any that call windowing specific
/// types or use the windowing system's event or event loop abstractions.
pub(super) trait AppHandlerCommon {
    fn handle_profile(
        &mut self,
        window_id: WindowIdentifier,
        end_profile: Option<WriteSignal<Option<Rc<Profile>>>>,
    );
    fn handle_theme_change(&mut self, theme: WindowSystemTheme);
    fn handle_menu_action(&mut self, action_id: MenuId);
    fn capture_window(&mut self, window_id: WindowIdentifier) -> Option<Capture>;
    fn publish_event(&mut self, event: AppEvent);
    fn take_expired_timers(&mut self) -> Vec<TimerToken>;
    fn remove_timer_instance(&mut self, token: &TimerToken) -> Option<Timer>;
    fn window_handle_for_window_id(
        &mut self,
        window_id: &WindowIdentifier,
    ) -> Option<&mut WindowHandle>;
}

/// Contains the crate-facing methods floem may call on ApplicationHandle. This is the set of methods that were
/// formerly `pub(crate)` when there was only one implementation of `ApplicationHandle`, exported by a `pub(crate)`
/// trait only to the rest of Floem.
#[allow(private_bounds)]
pub(crate) trait AppHandlerInternalAPI: AppHandlerCommon {
    /// The windowing system's abstraction for an event loop
    type WindowingSystemEventLoop: ?Sized;
    /// The windowing system's abstraction for an event that happened on a window which can be responded it.
    type WindowingSystemWindowEvent;

    fn handle_window_creation(
        &mut self,
        creation: WindowCreation,
        event_loop: &Self::WindowingSystemEventLoop,
    );

    fn new(config: AppConfig) -> Self;
    fn new_window(
        &mut self,
        event_loop: &Self::WindowingSystemEventLoop,
        view_fn: Box<dyn FnOnce(WindowIdentifier) -> Box<dyn View>>,
        override_theme: Option<WindowSystemTheme>,
        config: WindowConfig,
    );

    fn idle(&mut self) {
        let ext_events = { std::mem::take(&mut *EXT_EVENT_HANDLER.queue.lock()) };
        for trigger in ext_events {
            trigger.notify();
        }
        self.handle_updates_for_all_windows();
    }

    fn handle_updates_for_all_windows(&mut self);
    fn handle_timer(&mut self, event_loop: &Self::WindowingSystemEventLoop);
    fn handle_user_event(&mut self, event_loop: &Self::WindowingSystemEventLoop, event: UserEvent);
    fn handle_window_event(
        &mut self,
        window_id: WindowIdentifier,
        event: Self::WindowingSystemWindowEvent,
        event_loop: &Self::WindowingSystemEventLoop,
    );
}

/// Implementation logic for `ApplicationHandler` - this is the set of *internal* calls which must be implemented differently
/// for different windowing systems.
pub(super) trait AppHandlerImpl: AppHandlerInternalAPI {
    fn close_window(
        &mut self,
        window_id: WindowIdentifier,
        event_loop: &Self::WindowingSystemEventLoop,
    );

    fn request_timer(&mut self, timer: Timer, event_loop: &Self::WindowingSystemEventLoop);
    fn remove_timer(&mut self, timer: &TimerToken, event_loop: &Self::WindowingSystemEventLoop);
    fn fire_timer(&mut self, event_loop: &Self::WindowingSystemEventLoop);
    fn handle_gpu_resource_update(&mut self, window_id: WindowIdentifier);
    fn handle_exit(&mut self, event_loop: &Self::WindowingSystemEventLoop);

    fn handle_update_event(&mut self, event_loop: &Self::WindowingSystemEventLoop) {
        let events = crate::app_events::retreive_app_update_events();
        for event in events {
            match event {
                AppUpdateEvent::NewWindow { window_creation } => {
                    self.handle_window_creation(window_creation, event_loop)
                }
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
                AppUpdateEvent::ThemeChanged { theme } => self.handle_theme_change(theme),
            }
        }
    }

    fn internal_handle_user_event(
        &mut self,
        event_loop: &Self::WindowingSystemEventLoop,
        event: UserEvent,
    ) {
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

    fn internal_handle_timer(&mut self, event_loop: &Self::WindowingSystemEventLoop) {
        let tokens: Vec<TimerToken> = self.take_expired_timers();
        if !tokens.is_empty() {
            for token in tokens {
                if let Some(timer) = self.remove_timer_instance(&token) {
                    (timer.action)(token);
                }
            }
            self.handle_updates_for_all_windows();
        }
        self.fire_timer(event_loop);
    }
}
